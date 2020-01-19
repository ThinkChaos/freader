use actix_web::dev::HttpServiceFactory;
use actix_web::{web, HttpResponse};
use futures::future;
use serde::Serialize;

use crate::prelude::*;


pub fn service() -> impl HttpServiceFactory {
    web::resource("/user-info").route(web::get().to(get))
}


#[derive(Debug, Serialize)]
struct Response<'a> {
    #[serde(rename = "userId")]
    user_id: &'a str,
    #[serde(rename = "userName")]
    username: &'a str,
    #[serde(rename = "userProfileId")]
    profile_id: &'a str,
    #[serde(rename = "userEmail")]
    email: &'a str,
    #[serde(rename = "isBloggerUser")]
    is_blogger_user: bool,
    #[serde(rename = "signupTimeSec")]
    signup_time_sec: u8,
    // #[serde(rename = "publicUserName")]
    // public_user_name: &'a str,
    #[serde(rename = "isMultiLoginEnabled")]
    is_multi_login_enabled: bool,
}


fn get(data: web::Data<AppData>) -> future::Ready<HttpResponse> {
    future::ready(HttpResponse::Ok().json(Response {
        user_id: "0",
        username: &data.cfg.auth_username,
        profile_id: "0",
        email: &format!("{}@{}", data.cfg.auth_username, data.cfg.http_host),
        is_blogger_user: false,
        signup_time_sec: 0,
        // public_user_name: "username",
        is_multi_login_enabled: true,
    }))
}
