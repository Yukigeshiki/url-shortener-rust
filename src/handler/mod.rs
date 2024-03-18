pub use health_check::*;
pub use url_add::*;

mod health_check;
mod url_add;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Success<T: serde::Serialize> {
    pub request_id: String,
    pub payload: T,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fail {
    pub request_id: String,
    pub error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to serialise payload to JSON: {0}")]
    Serialisation(String),

    #[error("Failed to get Redis connection: {0}")]
    RedisConnection(String),

    #[error("Failed to execute Redis Query: {0}")]
    RedisQuery(String),
}
