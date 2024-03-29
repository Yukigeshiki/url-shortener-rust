use redis::{Client, RedisResult};
use secrecy::{ExposeSecret, Secret};
use std::convert::{TryFrom, TryInto};

use serde_aux::prelude::deserialize_number_from_string;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub redis: RedisSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RedisSettings {
    // pub username: String,
    // pub password: Secret<String>,
    pub host: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

impl RedisSettings {
    pub fn get_redis_client(&self) -> RedisResult<Client> {
        let connection_url = format!("redis://@{}:{}", self.host.expose_secret(), self.port);
        Client::open(connection_url)
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_directory.join("base.yml")))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Development,
    Production,
}

impl Environment {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `local`, `development` or `production`."
            )),
        }
    }
}
