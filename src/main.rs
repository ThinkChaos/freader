#[macro_use]
extern crate diesel;

use actix::prelude::*;
use actix_web::{middleware, web, App, HttpServer};

mod auth;
mod config;
mod db;
mod models;
mod reader;
mod schema;
mod utils;

use config::Config;

pub struct AppData {
    cfg: Config,
    db: db::Helper,
}

impl AppData {
    pub fn new(cfg: Config) -> Result<Self, diesel::result::ConnectionError> {
        // Test DB connection now
        drop(db::Executor::connect(&cfg.sqlite_db)?);

        let sqlite_db = cfg.sqlite_db.clone();
        let db_pool = SyncArbiter::start(2, move || {
            db::Executor::connect(&sqlite_db).expect("DB connection failed")
        });

        Ok(AppData {
            cfg,
            db: db::Helper::new(db_pool),
        })
    }
}

fn main() -> Result<(), std::io::Error> {
    if let Err(err) = dotenv::from_filename("ggrrss.env") {
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

    let sys = actix::System::new("ggrrss");

    let data = web::Data::new(AppData::new(cfg.clone()).map_err(|err| {
        log::error!("Database connection error: {}", err);
        std::process::exit(2);
    }));

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

    server.start();

    sys.run()
}
