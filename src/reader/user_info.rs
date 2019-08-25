use actix_web::dev::HttpServiceFactory;
use actix_web::{web, HttpResponse, Result};
use serde::Serialize;


pub fn service() -> impl HttpServiceFactory {
    web::resource("/user-info").route(web::get().to(get))
}


#[derive(Debug, Serialize)]
struct Response {
    #[serde(rename = "userId")]
    user_id: &'static str,
    #[serde(rename = "userName")]
    username: &'static str,
    #[serde(rename = "userProfileId")]
    profile_id: &'static str,
    #[serde(rename = "userEmail")]
    email: &'static str,
    #[serde(rename = "isBloggerUser")]
    is_blogger_user: bool,
    #[serde(rename = "signupTimeSec")]
    signup_time_sec: u8,
    // #[serde(rename = "publicUserName")]
    // public_user_name: &'static str,
    #[serde(rename = "isMultiLoginEnabled")]
    is_multi_login_enabled: bool,
}


fn get() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Response {
        user_id: "0",
        username: "User",
        profile_id: "0",
        email: "noone@localhost",
        is_blogger_user: false,
        signup_time_sec: 0,
        // public_user_name: "username",
        is_multi_login_enabled: true,
    }))
}
