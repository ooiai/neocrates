//! Custom Axum extractors with enhanced error handling
//!
//! This module provides custom JSON extractors for Axum that offer better error handling
//! and logging compared to the default `Json` extractor.
//!
//! # Available Extractors
//!
//! - [`LoggedJson`]: Logs deserialization errors and returns a generic error response
//! - [`DetailedJson`]: Provides detailed, structured error responses for different error types
//!
//! # Usage
//!
//! Replace `axum::Json` with `LoggedJson` or `DetailedJson` in your route handlers:
//!
//! ```rust,ignore
//! use neocrates::axum::{Router, routing::post};
//! use neocrates::helper::core::axum_extractor::LoggedJson;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CreateUserRequest {
//!     name: String,
//!     email: String,
//! }
//!
//! async fn create_user(LoggedJson(payload): LoggedJson<CreateUserRequest>) -> String {
//!     format!("Created user: {} ({})", payload.name, payload.email)
//! }
//!
//! let app = Router::new().route("/users", post(create_user));
//! ```

use crate::axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use crate::serde::de::DeserializeOwned;
use crate::{serde_json, tracing};

/// 自定义 JSON extractor，用于打印反序列化错误
///
/// 这个 extractor 会在 JSON 反序列化失败时记录详细的错误信息，
/// 并返回更友好的错误响应。当反序列化失败时，会：
/// 1. 使用 `tracing::error!` 记录详细的错误信息
/// 2. 返回 422 Unprocessable Entity 状态码
/// 3. 返回包含错误信息的 JSON 响应
///
/// # Response Format
///
/// 失败时返回的 JSON 格式：
/// ```json
/// {
///   "error": "JSON deserialization failed",
///   "message": "详细的错误描述..."
/// }
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use neocrates::axum::{Router, routing::post};
/// use neocrates::helper::core::axum_extractor::LoggedJson;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
/// }
///
/// async fn create_user(LoggedJson(payload): LoggedJson<CreateUser>) -> String {
///     format!("Created user: {}", payload.name)
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// ```
///
/// # When to Use
///
/// 使用 `LoggedJson` 当你需要：
/// - 简单的错误日志记录
/// - 统一的错误响应格式
/// - 快速替换标准的 `Json` extractor
pub struct LoggedJson<T>(pub T);

impl<S, T> FromRequest<S> for LoggedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(LoggedJson(value)),
            Err(rejection) => {
                let error_message = format!("JSON 反序列化失败: {:?}", rejection);
                tracing::error!("{}", error_message);

                // 返回详细的错误信息
                let response = (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({
                        "error": "JSON deserialization failed",
                        "message": error_message,
                    })),
                )
                    .into_response();

                Err(response)
            }
        }
    }
}

/// 自定义 JSON extractor，提供更详细的错误信息
///
/// 与 `LoggedJson` 类似，但会解析具体的错误类型并返回更结构化的错误响应。
/// 根据不同的错误类型返回相应的 HTTP 状态码和详细的错误信息。
///
/// # Error Types and Status Codes
///
/// - `JsonDataError`: 422 Unprocessable Entity - 数据格式不匹配（如类型错误、缺失字段等）
/// - `JsonSyntaxError`: 400 Bad Request - JSON 语法错误
/// - `MissingJsonContentType`: 415 Unsupported Media Type - 缺少正确的 Content-Type 头
/// - `BytesRejection`: 500 Internal Server Error - 无法读取请求体
/// - 其他错误: 400 Bad Request - 未知错误
///
/// # Response Format
///
/// 失败时返回的 JSON 格式：
/// ```json
/// {
///   "error": "json_data_error",
///   "message": "无效的 JSON 数据: missing field `email`",
///   "status": 422
/// }
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use neocrates::axum::{Router, routing::{post, put}};
/// use neocrates::helper::core::axum_extractor::DetailedJson;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UpdateUser {
///     id: i64,
///     name: Option<String>,
///     email: Option<String>,
/// }
///
/// async fn update_user(DetailedJson(payload): DetailedJson<UpdateUser>) -> String {
///     format!("Updated user: {}", payload.id)
/// }
///
/// let app = Router::new().route("/users", put(update_user));
/// ```
///
/// # When to Use
///
/// 使用 `DetailedJson` 当你需要：
/// - 详细的错误类型区分
/// - 不同错误类型对应不同的 HTTP 状态码
/// - 结构化的错误响应，便于前端处理
/// - 更好的 API 调试体验
pub struct DetailedJson<T>(pub T);

impl<S, T> FromRequest<S> for DetailedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(DetailedJson(value)),
            Err(rejection) => {
                let (status, error_type, message) = match rejection {
                    JsonRejection::JsonDataError(err) => (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "json_data_error",
                        format!("无效的 JSON 数据: {}", err),
                    ),
                    JsonRejection::JsonSyntaxError(err) => (
                        StatusCode::BAD_REQUEST,
                        "json_syntax_error",
                        format!("JSON 语法错误: {}", err),
                    ),
                    JsonRejection::MissingJsonContentType(err) => (
                        StatusCode::UNSUPPORTED_MEDIA_TYPE,
                        "missing_content_type",
                        format!("缺少 Content-Type: application/json 请求头: {}", err),
                    ),
                    JsonRejection::BytesRejection(err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "bytes_rejection",
                        format!("无法读取请求体: {}", err),
                    ),
                    _ => (
                        StatusCode::BAD_REQUEST,
                        "unknown_error",
                        format!("未知错误: {:?}", rejection),
                    ),
                };

                tracing::error!("JSON 提取失败 [{}]: {}", error_type, message);

                let response = (
                    status,
                    Json(serde_json::json!({
                        "error": error_type,
                        "message": message,
                        "status": status.as_u16(),
                    })),
                )
                    .into_response();

                Err(response)
            }
        }
    }
}
