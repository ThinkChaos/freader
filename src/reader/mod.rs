use actix_web::web;
use actix_web::dev::HttpServiceFactory;

mod subscription;
mod utils;
mod user_info;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/reader/api/0")
        .wrap(utils::RequireAuth)
        .service(subscription::service())
        .service(user_info::service())
}
