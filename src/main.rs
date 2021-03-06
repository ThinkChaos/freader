#[macro_use]
extern crate diesel;

use actix::Actor;
use actix_web::{middleware, web, App, HttpServer};

pub mod appdata;
pub mod auth;
pub mod config;
pub mod db;
pub mod feed_manager;
pub mod opml;
pub mod prelude;
pub mod reader;
pub mod updater;
pub mod utils;

use feed_manager::FeedManager;
use prelude::*;
use updater::Updater;

const ENV_FILENAME: &str = "freader.env";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match dotenv::from_filename(ENV_FILENAME) {
        Ok(_) => {
            log::debug!("Read {}", ENV_FILENAME);
        }
        Err(dotenv::Error::Io(io_err)) if io_err.kind() == std::io::ErrorKind::NotFound => (),
        Err(err) => {
            eprintln!("Could not parse {}: {}", ENV_FILENAME, err);
            std::process::exit(1);
        }
    }

    env_logger::init();

    let cfg = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            log::error!("Invalid config: {}", err);
            std::process::exit(1);
        }
    };

    let db = match db::Helper::new(&cfg) {
        Ok(db) => db,
        Err(err) => {
            log::error!("Database connection error: {}", err);
            std::process::exit(2);
        }
    };

    let feed_manager = FeedManager::new(db.clone());

    let updater = Updater::new(db.clone(), feed_manager.clone());

    let data = web::Data::new(AppData::new(cfg.clone(), db, feed_manager));

    if let Some(ecode) = handle_cli_args(&data).await? {
        std::process::exit(ecode);
    }

    // Create the HTTP server
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(auth::service())
            .service(reader::service())
            .default_service(web::route().to(
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
            match server.bind((cfg.http_host.as_str(), cfg.http_port)) {
                Ok(server) => server,
                Err(err) => {
                    log::error!(
                        "Could not bind to address '{}:{}': {}",
                        cfg.http_host,
                        cfg.http_port,
                        err
                    );
                    std::process::exit(2);
                }
            }
        };
    }

    let server = server.workers(2);

    log::info!("Listening on: http://{}:{}", cfg.http_host, cfg.http_port);
    for (addr, scheme) in server.addrs_with_scheme() {
        log::debug!("Using address: {}://{}", scheme, addr);
    }

    updater.start();

    server.run().await
}

/// Parse and handle CLI args.
///
/// Returns an exit code if the program should stop,
/// otherwise returns `None`.
async fn handle_cli_args(data: &AppData) -> std::io::Result<Option<i32>> {
    let mut args = std::env::args().into_iter().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(Some(0));
            }
            "--import" => {
                let file = match args.next() {
                    Some(x) => x,
                    None => {
                        eprintln!("Missing value for {}", arg);
                        return Ok(Some(1));
                    }
                };

                opml::import(&file, &mut data.db.clone()).await?;
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                print_usage();
                return Ok(Some(1));
            }
        }
    }

    Ok(None)
}

fn print_usage() {
    println!("USAGE: freader [-h | --help] [--import OPML]");
}
