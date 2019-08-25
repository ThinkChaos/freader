#[macro_use]
extern crate diesel;

use actix::prelude::*;
use actix_web::{middleware, web, App, HttpServer};

mod auth;
mod db;
mod models;
mod reader;
mod schema;
mod utils;

pub struct AppData {
    secret: String,
    db: db::Helper,
}

impl AppData {
    pub fn new(secret: &str, db_connspec: &str) -> Result<Self, diesel::result::ConnectionError> {
        // Test DB connection now
        drop(db::Executor::connect(db_connspec)?);

        let db_connspec = db_connspec.to_owned();

        let db_pool = SyncArbiter::start(2, move || {
            db::Executor::connect(&db_connspec).expect("DB connection failed")
        });

        Ok(AppData {
            secret: secret.to_owned(),
            db: db::Helper::new(db_pool),
        })
    }
}

fn main() -> Result<(), std::io::Error> {
    env_logger::from_env(
        env_logger::Env::default().default_filter_or("actix_web=debug,ggrrss=trace"),
    )
    .init();

    let sys = actix::System::new("ggrrss");

    let secret = "the-secret";
    let db_connspec = "file:ggrrss.sqlite";

    let data = web::Data::new(
        AppData::new(secret, db_connspec)
            .map_err(|err| {
                log::error!("Databse connection error: {}", err);
                err
            })
            .unwrap(),
    );

    // Start the HTTP server
    let mut server = HttpServer::new(move || {
        App::new()
            .register_data(data.clone())
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
            server.listen(l)?
        } else {
            server.bind("0.0.0.0:8088")?
        };
    }

    let server = server.workers(2);
    server.start();

    log::info!("Started http server: http://localhost:8088");

    sys.run()
}
