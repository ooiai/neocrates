use std::{fmt::Display, panic::Location};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

// 1. Common success status codes (2xx):
// - 200 for general success responses (GET/PUT/PATCH)
// - 201 for successful creation (POST)
// - 204 for no content responses (DELETE)
// - 202 for asynchronous task acceptance

// 2. Redirect status codes (3xx):
// - 301 for permanent resource relocation
// - 302 for temporary redirects
// - 304 for not modified, use cache
// - 307/308 for redirects preserving HTTP method

// 3. Client error status codes (4xx):
// - 400 for malformed requests (parameter errors)
// - 401 for unauthenticated requests (not logged in)
// - 403 for unauthorized access (logged in but insufficient permissions)
// - 404 for resource not found
// - 409 for resource conflicts (duplicate creation)

// 4. Server error status codes (5xx):
// - 500 for internal server errors (unexpected exceptions)
// - 502 for gateway errors (upstream service exceptions)
// - 503 for service unavailable (maintenance or overload)
// - 504 for gateway timeout (upstream service timeout)

// 5. Special purpose status codes:
// - 422 for business rule validation failures
// - 429 for request rate limits
// - 418 for fun easter egg responses

// 6. RESTful API common scenarios:
// - GET success returns 200
// - POST creation success returns 201
// - PUT/PATCH update success returns 200
// - DELETE success returns 204

// 7. Error handling principles:
// - Use 4xx series for client errors
// - Use 5xx series for server errors
// - Provide clear error messages and handling suggestions
// - Maintain uniform error response format

// 8. Status code selection criteria:
// - Prioritize semantically matching status codes
// - Maintain consistency within the application
// - Avoid uncommon status codes
// - Consider client compatibility

// 9. Cache-related status codes:
// - 304 used with ETag/Last-Modified
// - Cache control for GET requests
// - Use caching to improve performance

// 10. Security-related status codes:
// - 401 for authentication failures
// - 403 for permission verification failures
// - 429 for security rate limiting
// - 451 for legally prohibited access

pub type AppResult<T> = std::result::Result<T, AppError>;

// System error code enumeration
#[derive(Error, Debug)]
pub enum AppError {
    // Client errors (4xx)
    #[error("{0}")]
    ValidationError(String), // Parameter validation failure
    #[error("Unauthorized")]
    Unauthorized, // Not logged in or invalid token
    #[error("Token Expired")]
    TokenExpired,
    #[error("Forbidden")]
    Forbidden, // Insufficient permissions
    #[error("Resource not found: {0}")]
    NotFound(String), // Resource doesn't exist
    #[error("Request conflict: {0}")]
    Conflict(String), // Resource conflict
    #[error("{0}")]
    ClientError(String), // General client error
    #[error("{0}")]
    ClientDataError(String), // General client data error

    #[error("Business rule validation failed: {0}")]
    UnprocessableEntity(String), // 422: Business rule validation
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String), // 429: Rate limit exceeded
    #[error("{0}")]
    EasterEgg(String), // 418: Fun easter egg responses

    // Server errors (5xx)
    #[error("Database error: {0}")]
    DbError(String), // Database error
    #[error("Redis error: {0}")]
    RedisError(String), // Redis error
    #[error("Message queue error: {0}")]
    MqError(String), // Message queue error
    #[error("External service error: {0}")]
    ExternalError(String), // External service call error
    #[error("Internal server error")]
    Internal(String), // Other internal errors

    #[error("{1}")]
    DataError(u32, String), // Custom business code and error message

    #[error("{0}")]
    JsonError(String), // JSON serialization error
}

// API response structure
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: u32,       // Business status code
    pub message: String, // Error message
    pub data: Option<T>, // Response data
}

// Error code and HTTP status code mapping
/// HTTP status code and business code mappings for application errors
impl AppError {
    // HTTP status code constants
    const HTTP_BAD_REQUEST: StatusCode = StatusCode::BAD_REQUEST; // 400
    const HTTP_UNAUTHORIZED: StatusCode = StatusCode::UNAUTHORIZED; // 401
    const HTTP_FORBIDDEN: StatusCode = StatusCode::FORBIDDEN; // 403
    const HTTP_NOT_FOUND: StatusCode = StatusCode::NOT_FOUND; // 404
    const HTTP_CONFLICT: StatusCode = StatusCode::CONFLICT; // 409
    const HTTP_UNPROCESSABLE_ENTITY: StatusCode = StatusCode::UNPROCESSABLE_ENTITY; // 422
    const HTTP_TOO_MANY_REQUESTS: StatusCode = StatusCode::TOO_MANY_REQUESTS; // 429
    const HTTP_IM_A_TEAPOT: StatusCode = StatusCode::IM_A_TEAPOT; // 418
    const EXPECTATION_FAILED: StatusCode = StatusCode::EXPECTATION_FAILED; // 417
    const HTTP_INTERNAL_ERROR: StatusCode = StatusCode::INTERNAL_SERVER_ERROR; // 500

    // Business error code constants
    const BIZ_VALIDATION_ERROR: u32 = 400001;
    const BIZ_UNAUTHORIZED: u32 = 400002;
    const BIZ_FORBIDDEN: u32 = 400003;
    const BIZ_NOT_FOUND: u32 = 400004;
    const BIZ_CONFLICT: u32 = 400005;
    const BIZ_CLIENT_ERROR: u32 = 400006;
    const BIZ_DATA_ERROR: u32 = 400007;
    const BIZ_TOKEN_EXPIRED: u32 = 400008;
    const BIZ_DB_ERROR: u32 = 500001;
    const BIZ_REDIS_ERROR: u32 = 500002;
    const BIZ_MQ_ERROR: u32 = 500003;
    const BIZ_EXTERNAL_ERROR: u32 = 500004;
    const BIZ_INTERNAL_ERROR: u32 = 500000;
    const BIZ_UNPROCESSABLE_ENTITY: u32 = 400100; // Business validation errors
    const BIZ_RATE_LIMIT: u32 = 400101; // Rate limiting errors
    const BIZ_EASTER_EGG: u32 = 400102; // Easter egg responses

    // Business data errors - Expanded categories
    // 410000-410099: Data existence errors
    pub const BIZ_DATA_EXISTS: u32 = 410000; // Data already exists
    pub const BIZ_DATA_DUPLICATE: u32 = 410001; // Duplicate entry found
    pub const BIZ_DATA_NOT_FOUND: u32 = 410002; // Data not found/doesn't exist
    pub const BIZ_DATA_DELETED: u32 = 410003; // Data has been deleted
    pub const BIZ_DATA_ARCHIVED: u32 = 410004; // Data is archived
    pub const BIZ_DATA_OUTDATED: u32 = 410005; // Data version is outdated

    // 410100-410199: JSON serialization errors
    pub const BIZ_JSON_ERROR: u32 = 410100; // JSON serialization error

    /// Maps application errors to HTTP status codes
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 4xx Client Errors
            Self::ValidationError(_) => Self::HTTP_BAD_REQUEST,
            Self::Unauthorized => Self::HTTP_UNAUTHORIZED,
            Self::TokenExpired => Self::HTTP_UNAUTHORIZED,
            Self::Forbidden => Self::HTTP_FORBIDDEN,
            Self::NotFound(_) => Self::HTTP_NOT_FOUND,
            Self::Conflict(_) => Self::HTTP_CONFLICT,
            Self::UnprocessableEntity(_) => Self::HTTP_UNPROCESSABLE_ENTITY,
            Self::RateLimit(_) => Self::HTTP_TOO_MANY_REQUESTS,
            Self::EasterEgg(_) => Self::HTTP_IM_A_TEAPOT,
            Self::Internal(_) => Self::HTTP_INTERNAL_ERROR,
            Self::ClientError(_) => Self::EXPECTATION_FAILED,
            Self::DataError(_, _) => Self::HTTP_CONFLICT, // All data errors use HTTP 409
            // 4xx HTTP_BAD_REQUEST - Return 400 for all
            _ => Self::HTTP_BAD_REQUEST,
        }
    }

    /// Maps application errors to business error codes
    pub fn business_code(&self) -> u32 {
        match self {
            // 4xx Client Errors
            Self::ValidationError(_) => Self::BIZ_VALIDATION_ERROR,
            Self::Unauthorized => Self::BIZ_UNAUTHORIZED,
            Self::TokenExpired => Self::BIZ_TOKEN_EXPIRED,
            Self::Forbidden => Self::BIZ_FORBIDDEN,
            Self::NotFound(_) => Self::BIZ_NOT_FOUND,
            Self::Conflict(_) => Self::BIZ_CONFLICT,
            Self::UnprocessableEntity(_) => Self::BIZ_UNPROCESSABLE_ENTITY,
            Self::RateLimit(_) => Self::BIZ_RATE_LIMIT,
            Self::EasterEgg(_) => Self::BIZ_EASTER_EGG,
            Self::ClientError(_) => Self::BIZ_CLIENT_ERROR,
            Self::ClientDataError(_) => Self::BIZ_DATA_ERROR,
            Self::DataError(code, _) => *code, // Use the custom business code from DataError
            // 5xx Server Errors
            Self::DbError(_) => Self::BIZ_DB_ERROR,
            Self::RedisError(_) => Self::BIZ_REDIS_ERROR,
            Self::MqError(_) => Self::BIZ_MQ_ERROR,
            Self::ExternalError(_) => Self::BIZ_EXTERNAL_ERROR,
            Self::Internal(_) => Self::BIZ_INTERNAL_ERROR,
            // Business data errors
            // Self::DataExtis(_) => Self::BIZ_DATA_EXTIS,
            //
            Self::JsonError(_) => Self::BIZ_JSON_ERROR,
        }
    }

    /// Returns a user-friendly error message
    pub fn message(&self) -> String {
        match self {
            Self::UnprocessableEntity(msg) => msg.to_string(),
            Self::RateLimit(msg) => format!("Rate limit exceeded: {}", msg),
            Self::EasterEgg(msg) => format!("Easter egg: {}", msg),
            Self::ValidationError(msg) => msg.to_string(),
            Self::Unauthorized => "Unauthorized access".to_string(),
            Self::TokenExpired => "Token expired".to_string(),
            Self::Forbidden => "Access forbidden".to_string(),
            Self::NotFound(msg) => msg.to_string(),
            Self::Conflict(msg) => msg.to_string(),
            Self::DbError(e) => format!("Database error: {}", e),
            Self::RedisError(e) => format!("Cache error: {}", e),
            Self::MqError(e) => format!("Message queue error: {}", e),
            Self::ExternalError(e) => format!("External service error: {}", e),
            Self::Internal(e) => format!("Internal server error: {}", e),
            Self::ClientError(msg) => msg.to_string(),
            Self::ClientDataError(msg) => msg.to_string(),
            Self::DataError(_, msg) => msg.to_string(),
            Self::JsonError(msg) => format!("JSON serialization error: {}", msg),
        }
    }
}

// Implement response conversion
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let response = ApiResponse {
            code: self.business_code(),
            message: self.to_string(),
            data: None::<()>,
        };
        // Log the response
        tracing::error!(
            "...App Error...: code:{:?} message:{:?} self:{:?}",
            response.code,
            response.message,
            self
        );
        (status, Json(response)).into_response()
    }
}

impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        tracing::warn!("Parameter validation failed: {:?}", err);
        let message = err
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_messages: Vec<String> = errors
                    .iter()
                    .filter_map(|error| error.message.as_ref().map(|m| m.to_string()))
                    .collect();
                format!("{}: {}", field, error_messages.join(", "))
            })
            .collect::<Vec<String>>()
            .join("; ");

        AppError::ValidationError(format!("Parameter validation failed: {}", message))
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        tracing::error!("Database error: {}", err);
        AppError::DbError(err.to_string())
    }
}

impl From<deadpool_diesel::PoolError> for AppError {
    fn from(err: deadpool_diesel::PoolError) -> Self {
        tracing::error!("Deadpool_diesel Database error: {}", err);
        AppError::DbError(err.to_string())
    }
}

#[track_caller]
pub fn msg_with_location<M: Display>(msg: M) -> String {
    let loc = Location::caller();
    format!("{}:{} {}", loc.file(), loc.line(), msg)
}

impl AppError {
    #[track_caller]
    pub fn client_here<M: Display>(msg: M) -> Self {
        AppError::ClientError(msg_with_location(msg))
    }

    #[track_caller]
    pub fn data_here<M: Display>(msg: M) -> Self {
        AppError::ClientDataError(msg_with_location(msg))
    }

    #[track_caller]
    pub fn conflict_here<M: Display>(msg: M) -> Self {
        AppError::Conflict(msg_with_location(msg))
    }

    #[track_caller]
    pub fn not_found_here<M: Display>(msg: M) -> Self {
        AppError::NotFound(msg_with_location(msg))
    }
}

pub trait AppResultExt<T, E> {
    #[track_caller]
    fn client_context(self) -> AppResult<T>
    where
        E: Display;

    #[track_caller]
    fn context_msg(self, msg: impl Into<String>) -> AppResult<T>
    where
        E: Display;
}

impl<T, E> AppResultExt<T, E> for Result<T, E> {
    #[track_caller]
    fn client_context(self) -> AppResult<T>
    where
        E: Display,
    {
        self.map_err(|e| AppError::client_here(e))
    }

    #[track_caller]
    fn context_msg(self, msg: impl Into<String>) -> AppResult<T>
    where
        E: Display,
    {
        self.map_err(|e| AppError::client_here(format!("{} - {}", msg.into(), e)))
    }
}

// let chat_model: ChatModel = AgentChatService::get_agent_and_model(&app_state, pctx.aid)
//     .await
//     .map_err(AppError::client_here)?; // 等价于 |e| AppError::client_here(e)

// use crate::response::error::AppResultExt;

// let chat_model = AgentChatService::get_agent_and_model(&app_state, pctx.aid)
//     .await
//     .client_context()?; // 自动附带调用行号

// impl Error for diesel::result::Error {
//     fn as_infra_error(&self) -> InfraError {
//         match self {
//             diesel::result::Error::NotFound => InfraError::NotFound,
//             _ => InfraError::InternalServerError,
//         }
//     }
// }

// impl Error for deadpool_diesel::PoolError {
//     fn as_infra_error(&self) -> InfraError {
//         InfraError::InternalServerError
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_error_handling_examples() {
//         // 1. Business validation error (422)
//         let validation_error = AppError::UnprocessableEntity("用户名已存在".to_string());
//         assert_eq!(
//             validation_error.status_code(),
//             StatusCode::UNPROCESSABLE_ENTITY
//         );
//         assert_eq!(validation_error.message(), "用户名已存在");

//         // 2. Rate limiting error (429)
//         let rate_limit_error = AppError::RateLimit("每分钟最多请求60次".to_string());
//         assert_eq!(
//             rate_limit_error.status_code(),
//             StatusCode::TOO_MANY_REQUESTS
//         );
//         assert_eq!(
//             rate_limit_error.message(),
//             "请求频率超限: 每分钟最多请求60次"
//         );

//         // 3. Easter egg response (418)
//         let easter_egg = AppError::EasterEgg("我是一个可爱的茶壶".to_string());
//         assert_eq!(easter_egg.status_code(), StatusCode::IM_A_TEAPOT);
//         assert_eq!(easter_egg.message(), "趣味彩蛋: 我是一个可爱的茶壶");
//     }

//     // 实际使用示例
//     async fn example_handler() -> Result<String, AppError> {
//         // 1. 业务规则验证
//         if !is_valid_business_rule() {
//             return Err(AppError::UnprocessableEntity(
//                 "输入数据不符合业务规则".to_string(),
//             ));
//         }

//         // 2. 频率限制检查
//         if is_rate_limited() {
//             return Err(AppError::RateLimit("请稍后再试".to_string()));
//         }

//         // 3. 趣味彩蛋
//         if is_april_first() {
//             return Err(AppError::EasterEgg("今天是愚人节!".to_string()));
//         }

//         Ok("处理成功".to_string())
//     }

//     // Mock functions
//     fn is_valid_business_rule() -> bool {
//         true
//     }
//     fn is_rate_limited() -> bool {
//         false
//     }
//     fn is_april_first() -> bool {
//         false
//     }
// }
