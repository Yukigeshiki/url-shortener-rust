use std::fmt::{Display, Formatter};

pub use health_check::*;
pub use url_add::*;
pub use url_delete::*;
pub use url_redirect::*;

mod health_check;
mod url_add;
mod url_delete;
mod url_redirect;

#[allow(clippy::module_name_repetitions)]
pub type HandlerResult<T> = Result<T, Error>;

#[macro_export]
macro_rules! impl_json_display {
    ($strct:ty) => {
        impl Display for $strct {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!(
                    "{}",
                    serde_json::to_string(self).unwrap_or("Not available".into())
                ))
            }
        }
    };
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlResponsePayload {
    pub key: String,
    pub long_url: String,
    pub short_url: String,
}

impl_json_display!(UrlResponsePayload);

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

    #[error("Item with key '{0}' cannot be found")]
    NotFound(String),
}
