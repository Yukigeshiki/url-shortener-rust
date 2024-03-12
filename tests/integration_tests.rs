use std::net::TcpListener;

use once_cell::sync::Lazy;
use url_shortener::startup::run;
use url_shortener::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
}

async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server = run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp { address }
}

#[tokio::test]
async fn health_check_is_success() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(&format!("{}/health-check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}
