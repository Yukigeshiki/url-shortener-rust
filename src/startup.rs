use std::net::TcpListener;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use redis::Client;
use tracing_actix_web::TracingLogger;

use crate::handler::{health_check, url_add, url_delete, url_redirect, url_redirect_error};

#[allow(clippy::too_many_lines)]
pub fn run(listener: TcpListener, redis_client: Client) -> Result<Server, std::io::Error> {
    let redis_client = Data::new(redis_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .route("/health-check", web::get().to(health_check))
            .route("/", web::post().to(url_add))
            .route("/{key}", web::get().to(url_redirect))
            .route("/redirect-error/{msg}", web::get().to(url_redirect_error))
            .route("/{key}", web::delete().to(url_delete))
            .app_data(redis_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
