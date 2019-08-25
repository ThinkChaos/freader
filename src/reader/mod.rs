use actix_web::dev::HttpServiceFactory;
use actix_web::web;

mod subscription;
mod user_info;
mod utils;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/reader/api/0")
        .wrap(utils::RequireAuth)
        .service(subscription::service())
        .service(user_info::service())
}
