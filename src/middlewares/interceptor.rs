use axum::{
    body::{Body, Bytes},
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};
use url::form_urlencoded;

use crate::{
    middlewares::{
        ip::get_request_host,
        models::{AUTHORIZATION, AuthModel, BASIC, BEARER, CACHE_AUTH_TOKEN, MiddlewareConfig},
    },
    response::error::{AppError, AppResult},
};

/// The web global interceptor that can be used for all requests.
///
/// # Arguments
/// request - The incoming HTTP request.
/// next - The next middleware or handler in the chain.
///
/// Returns
/// A Response after processing the request.
///
pub async fn interceptor(
    config: &Arc<MiddlewareConfig>,
    mut request: Request,
    next: Next,
) -> Response {
    let token_store = &config.token_store;
    let ignore_urls = &config.ignore_urls;
    let prefix = &config.prefix;
    let pms_ignore_urls = &config.pms_ignore_urls;

    let (request_ip, uri) = get_request_host(&mut request);
    tracing::info!(
        "Middleware interceptor - client_ip: {} uri: {:?}",
        request_ip,
        uri
    );
    // Bypass middleware for URLs matching configured ignore prefixes
    if ignore_urls
        .iter()
        .any(|ignore_url| uri.starts_with(ignore_url))
    {
        return next.run(request).await;
    }
    // PMS (Permission Management System) ignore URLs
    if pms_ignore_urls
        .iter()
        .any(|ignore_url| uri.starts_with(ignore_url))
    {
        if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with(BASIC) {}
            }
        } else {
            tracing::warn!("Middleware Missing Authorization BASIC header");
            return AppError::Unauthorized.into_response();
        }
        return next.run(request).await;
    }
    // Support two token sources: Authorization header and accessToken query param
    let mut token_opt: Option<String> = None;
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with(BEARER) {
                token_opt = Some(auth_str[7..].to_string());
            }
        }
    }
    if token_opt.is_none() {
        if let Some(query) = request.uri().query() {
            let params: HashMap<_, _> = form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect();
            if let Some(token) = params.get("accessToken") {
                token_opt = Some(token.clone());
            }
        }
    }
    if let Some(token) = token_opt {
        let store_key = format!("{}{}{}", prefix, CACHE_AUTH_TOKEN, token);
        let auth_model: AuthModel = match crate::middlewares::token_store::store_get::<AuthModel>(
            token_store.as_ref(),
            &store_key,
        )
        .await
        {
            Ok(Some(m)) => m,
            Ok(None) => return AppError::TokenExpired.into_response(),
            Err(e) => {
                tracing::warn!("Middleware failed to fetch token from store: {}", e);
                return AppError::TokenExpired.into_response();
            }
        };
        tracing::warn!("Middleware extracted cache_token: {:?}", &auth_model);
        // TODO: Load admin role permission

        // TODO: Load agent role permission

        // Rewrite auth model into request extensions
        request.extensions_mut().insert(auth_model);
    } else {
        tracing::warn!(
            "Middleware Missing Authorization BEARER header and accessToken query param"
        );
        return AppError::Unauthorized.into_response();
    }
    // Read and modify the body
    let body_bytes = match read_and_print_body(&mut request).await {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    let modified_bytes = match modify_body(body_bytes, &mut request).await {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    *request.body_mut() = Body::from(modified_bytes);

    // next response
    let response = next.run(request).await;
    response
}

/// Read and print the request body
/// # Arguments
/// request - The incoming HTTP request.
///
/// Returns
/// The request body as Bytes.
async fn read_and_print_body(request: &mut Request) -> AppResult<Bytes> {
    let body = std::mem::replace(request.body_mut(), Body::empty());

    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| AppError::ClientError("Middleware Invalid request body".into()))?;

    // if let Ok(body_str) = String::from_utf8(bytes.to_vec()) {
    //     warn!("「read_and_print_body」Request body: {}", body_str);
    // }

    Ok(bytes)
}

/// Modify the request body.
///
/// # Arguments
/// bytes - The original request body as Bytes.
/// request - The incoming HTTP request.
///
/// Returns
/// The modified request body as Bytes.
async fn modify_body(bytes: Bytes, request: &mut Request) -> AppResult<Bytes> {
    if bytes.is_empty() {
        return Ok(bytes);
    }
    if let Ok(mut json) = serde_json::from_slice::<Value>(&bytes) {
        match &mut json {
            Value::Object(obj) => {
                insert_auth_fields(obj, request);
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    if let Value::Object(obj) = item {
                        insert_auth_fields(obj, request);
                    }
                }
            }
            _ => {
                tracing::warn!("Middleware Interceptor json is not object or array");
            }
        }

        let modified_bytes = serde_json::to_vec(&json)
            .map_err(|_| AppError::Internal("Middleware Interceptor JSON encode error".into()))?;
        return Ok(Bytes::from(modified_bytes));
    } else {
        tracing::warn!("Middleware Interceptor json parse failed");
    }

    Ok(bytes)
}

/// Insert authentication fields into the JSON object.
///
/// # Arguments
/// obj - The JSON object to modify.
/// request - The incoming HTTP request.
///
/// Returns
/// Nothing. The function modifies the JSON object in place.
fn insert_auth_fields(obj: &mut serde_json::Map<String, Value>, request: &mut Request) {
    match request.method().as_str() {
        "POST" => {
            if let Some(auth_model) = request.extensions().get::<AuthModel>() {
                obj.insert("creator".to_string(), json!(auth_model.uid));
                obj.insert("creator_by".to_string(), json!(auth_model.nickname));
                obj.insert("updater".to_string(), json!(auth_model.uid));
                obj.insert("updater_by".to_string(), json!(auth_model.nickname));
            } else {
                obj.insert("creator".to_string(), json!(0));
                obj.insert("creator_by".to_string(), json!("anonymous"));
                obj.insert("updater".to_string(), json!(0));
                obj.insert("updater_by".to_string(), json!("anonymous"));
            }
        }
        "PUT" => {
            if let Some(auth_model) = request.extensions().get::<AuthModel>() {
                obj.insert("updater".to_string(), json!(auth_model.uid));
                obj.insert("updater_by".to_string(), json!(auth_model.nickname));
            } else {
                obj.insert("updater".to_string(), json!(0));
                obj.insert("updater_by".to_string(), json!("anonymous"));
            }
        }
        _ => {}
    }
}
