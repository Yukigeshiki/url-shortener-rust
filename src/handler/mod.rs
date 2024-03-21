pub use health_check::*;
pub use url_add::*;
pub use url_delete::*;
pub use url_redirect::*;

mod health_check;
mod url_add;
mod url_delete;
mod url_redirect;

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
    #[error("Failed to serialise/deserialise payload: {0}")]
    Serialisation(String),

    #[error("Failed to get Redis connection: {0}")]
    RedisConnection(String),

    #[error("Failed to execute Redis Query: {0}")]
    RedisQuery(String),

    #[error("Item with key '{0}' not not exist")]
    NotFound(String),
}
