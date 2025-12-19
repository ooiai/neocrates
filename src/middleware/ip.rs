use axum::extract::Request;

/// 获取请求的 ip 和 uri
pub fn get_request_host(request: &Request) -> (String, String) {
    let real_ip = request.headers().get("x-real-ip");
    let mut request_ip = if real_ip.is_some() {
        real_ip
            .expect("Failed to get real IP")
            .to_str()
            .expect("Failed to convert real IP to string")
    } else {
        ""
    };
    if request_ip.is_empty() {
        request_ip = request
            .headers()
            .get("x-forwarded-for")
            .expect("Failed to get forwarded IP")
            .to_str()
            .expect("Failed to convert forwarded IP to string");
    }
    let uri = request.uri().path();
    (request_ip.to_string(), uri.to_string())
}
