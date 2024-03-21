use actix_web::http::header::CONTENT_TYPE;
use std::net::TcpListener;

use once_cell::sync::Lazy;
use url_shortener::configuration::get_configuration;
use url_shortener::startup::run;
use url_shortener::telemetry::{get_subscriber, init_subscriber};

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
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let redis_client = configuration
        .redis
        .get_redis_client()
        .expect("Failed to get Redis client.");

    let server = run(listener, redis_client).expect("Failed to bind address");
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

#[tokio::test]
async fn url_add_is_success() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .post(&format!("{}/", &app.address))
        .body(r#"{"longUrl": "https://codingchallenges.fyi/challenges/challenge-url-shortener/"}"#)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, res.status().as_u16());
}

#[tokio::test]
async fn url_redirect_is_success() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    client
        .post(&format!("{}/", &app.address))
        .body(r#"{"longUrl": "https://codingchallenges.fyi/challenges/challenge-url-shortener/"}"#)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    let res = client
        .get(&format!("{}/4168cd70", &app.address))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, res.status().as_u16());
    assert!(res.content_length().is_some());
}

#[tokio::test]
async fn url_redirect_not_found_is_failure() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(&format!("{}/4168cd7y", &app.address))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, res.status().as_u16());
}

#[tokio::test]
async fn url_delete_is_success() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    client
        .post(&format!("{}/", &app.address))
        .body(r#"{"longUrl": "https://codingchallenges.fyi/challenges/challenge-url-shortener/"}"#)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    let res = client
        .delete(&format!("{}/4168cd70", &app.address))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, res.status().as_u16());
    assert!(res.content_length().is_some());
}

#[tokio::test]
async fn url_delete_not_found_is_failure() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .delete(&format!("{}/4168cd7y", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, res.status().as_u16());
}
