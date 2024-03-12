use std::net::TcpListener;

use url_shortener::configuration::get_configuration;
use url_shortener::run;
use url_shortener::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("url_shortener".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let redis_client = configuration
        .redis
        .get_redis_client()
        .expect("Failed to get Redis client.");

    let port = configuration.application.port;
    let address = format!("{}:{}", configuration.application.host, port);
    tracing::info!("Application listening on port {}", port);
    let listener = TcpListener::bind(address)?;
    run(listener, redis_client)?.await?;
    Ok(())
}
