use actix::prelude::*;
use actix_web::{middleware, App, HttpServer};

mod auth;
mod reader;
mod utils;

#[derive(Clone)]
pub struct Data {
    secret: String,
}

fn main() -> Result<(), std::io::Error> {
    env_logger::from_env(
            env_logger::Env::default().default_filter_or("actix_web=debug,ggrrss=trace")
        )
        .init();

    let sys = actix::System::new("ggrrss");

    let data = Data {
        secret: "the-secret".to_string(),
    };

    // Start the HTTP server
    HttpServer::new(move || {
            App::new()
                .data(data.clone())
                .wrap(middleware::Compress::default())
                .wrap(middleware::Logger::default())
                .service(auth::service())
                .service(reader::service())
                .default_service(actix_web::web::route().to(|req: actix_web::HttpRequest, body: actix_web::web::Bytes| {
                    #[cfg(debug_assertions)]
                    utils::dump_request_and_body(&req, &body);

                    actix_web::HttpResponse::NotFound()
                }))
        })
        .bind("127.0.0.1:8088")?
        .start();

    println!("Started http server: http://localhost:8088");

    sys.run()
}
