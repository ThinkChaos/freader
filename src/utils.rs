use actix_web::HttpRequest;

pub trait HttpService = actix_service::Service<
    Request = actix_web::dev::ServiceRequest,
    Response = actix_web::dev::ServiceResponse<actix_http::body::Body>,
    Error = actix_web::Error,
>;

#[allow(dead_code)]
pub fn dump_request_and_body(req: &HttpRequest, body: &[u8]) {
    let body = std::str::from_utf8(body).unwrap_or("");

    log::debug!(
        "{:?}\n{}{}",
        req,
        body,
        if body.is_empty() { "" } else { "\n" }
    );
}
