use actix_web::{dev, http, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::AppData;

#[derive(Debug, Deserialize)]
struct LoginData {
    // #[serde(rename = "accountType")]
    // account_type: String,
    // service: String,
    // client: String,
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Passwd")]
    password: String,
    // output: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    #[serde(rename = "Auth")]
    token: String,
    // #[serde(rename = "SID")]
    // sid: String,
    // #[serde(rename = "LSID")]
    // lsid: String,
}

pub fn service() -> impl dev::HttpServiceFactory {
    web::scope("/accounts").route("/ClientLogin", web::post().to(login))
}

fn login(data: web::Data<AppData>, form: web::Form<LoginData>) -> HttpResponse {
    if form.password == data.secret {
        HttpResponse::Ok().json(LoginResponse {
            token: form.into_inner().password,
        })
    } else {
        HttpResponse::Forbidden()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(r#"{Error: "BadAuthentication"}"#)
    }
}
