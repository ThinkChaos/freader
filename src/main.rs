#![feature(trait_alias)]

#[macro_use]
extern crate diesel;

use actix_web::{middleware, web, App, HttpServer};

pub mod appdata;
pub mod auth;
pub mod config;
pub mod db;
pub mod models;
pub mod prelude;
pub mod reader;
pub mod schema;
pub mod utils;

use prelude::*;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    if let Err(err) = dotenv::from_filename("freader.env") {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    env_logger::init();

    let cfg = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            log::error!("Invalid config: {}", err);
            std::process::exit(1);
        }
    };

    let data = web::Data::new(match AppData::new(cfg.clone()) {
        Ok(data) => data,
        Err(err) => {
            log::error!("Database connection error: {}", err);
            std::process::exit(2);
        }
    });

    // Start the HTTP server
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(auth::service())
            .service(reader::service())
            .default_service(actix_web::web::route().to(
                |_req: actix_web::HttpRequest, _body: actix_web::web::Bytes| {
                    #[cfg(feature = "dev")]
                    utils::dump_request_and_body(&_req, &_body);

                    actix_web::HttpResponse::NotFound()
                },
            ))
    });

    // Bind server
    {
        #[cfg(not(feature = "dev"))]
        let listener: Option<std::net::TcpListener> = None;

        #[cfg(feature = "dev")]
        let listener = listenfd::ListenFd::from_env().take_tcp_listener(0)?; // for auto reloading

        server = if let Some(l) = listener {
            log::debug!("Ignoring configured host and port: using socket from systemfd");
            server.listen(l)?
        } else {
            server.bind((cfg.http_host.as_str(), cfg.http_port))?
        };
    }

    let server = server.workers(2);

    for (addr, scheme) in server.addrs_with_scheme() {
        log::info!("Listening on: {}://{}", scheme, addr);
    }

    server.run().await
}
