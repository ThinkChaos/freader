#[macro_use]
extern crate diesel;

use actix::prelude::*;
use actix_web::{middleware, App, HttpServer};

mod auth;
mod db;
mod models;
mod reader;
mod schema;
mod utils;

#[derive(Clone)]
pub struct Data {
    secret: String,
    db: Addr<db::Executor>,
}

fn main() -> Result<(), std::io::Error> {
    env_logger::from_env(
            env_logger::Env::default().default_filter_or("actix_web=debug,ggrrss=trace")
        )
        .init();

    let sys = actix::System::new("ggrrss");

    // Start 3 db executors
    let db = SyncArbiter::start(3, || {
        db::Executor::new("ggrrss.sqlite").expect("Failed to open db")
    });

    let data = Data {
        secret: "the-secret".to_string(),
        db,
    };

    // Start the HTTP server
    let mut server = HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(auth::service())
            .service(reader::service())
            .default_service(actix_web::web::route().to(|_req: actix_web::HttpRequest, _body: actix_web::web::Bytes| {
                #[cfg(feature="dev")]
                utils::dump_request_and_body(&_req, &_body);

                actix_web::HttpResponse::NotFound()
            }))
    });

    // Bind server
    #[cfg(not(feature="dev"))]
    let listener: Option<std::net::TcpListener> = None;

    #[cfg(feature="dev")]
    let listener = listenfd::ListenFd::from_env().take_tcp_listener(0)?; // for auto reloading

    server = if let Some(l) = listener {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8088")?
    };

    server.start();

    println!("Started http server: http://localhost:8088");

    sys.run()
}
