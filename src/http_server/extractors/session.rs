fn extract_cookie_as_string(request: &actix_web::HttpRequest, name: &str) -> String {
    match request.cookie(name) {
        Some(cookie) => cookie.value().to_owned(),
        None => "".to_owned()
    }
}