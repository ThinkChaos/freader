use actix_web::HttpRequest;

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

/// Ensure `url` is absolute.
///
/// If it is not, it is joined to `base`.
pub fn make_url_absolute(url: &str, base: &str) -> Result<String, &'static str> {
    if url.starts_with("http") {
        Ok(url.to_owned())
    } else {
        reqwest::Url::parse(&base)
            .and_then(|base| base.join(&url))
            .map(|url| url.to_string())
            .map_err(|_| "Invalid URL")
    }
}
