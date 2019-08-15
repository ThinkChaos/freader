use actix_web::HttpRequest;

#[allow(dead_code)]
pub fn dump_request_and_body(req: &HttpRequest, body: &[u8]) {
    let body = std::str::from_utf8(body).unwrap_or("");

    eprintln!("{:?}\n{}{}", req, body, if body.is_empty() { "" } else { "\n" });
}
