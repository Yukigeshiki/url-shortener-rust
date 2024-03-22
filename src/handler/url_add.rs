use std::fmt::{Display, Formatter};

use actix_web::{web, HttpResponse};
use redis::{Client, Commands, ErrorKind};
use sha256::digest;
use tracing_actix_web::RequestId;

use crate::handler::{Error, Fail, Success, UrlResponsePayload};
use crate::impl_json_display;
use crate::parser::LongUrl;

struct UrlNew {
    pub long_url: LongUrl,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct UrlJsonData {
    long_url: String,
}

impl_json_display!(UrlJsonData);

impl TryFrom<UrlJsonData> for UrlNew {
    type Error = String;

    fn try_from(value: UrlJsonData) -> Result<Self, Self::Error> {
        let long_url = LongUrl::parse(value.long_url)?;
        Ok(Self { long_url })
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "URL add route",
    skip(request_id, json_data, redis_client),
    fields(
        data = % json_data
    )
)]
pub async fn url_add(
    request_id: RequestId,
    json_data: web::Json<UrlJsonData>,
    redis_client: web::Data<Client>,
) -> HttpResponse {
    let url: UrlNew = match json_data.0.try_into() {
        Ok(v) => v,
        Err(err) => {
            return HttpResponse::BadRequest().json(Fail {
                request_id: request_id.to_string(),
                error: err,
            });
        }
    };

    match add(&redis_client, url.long_url.as_ref()).await {
        Ok(payload) => {
            tracing::info!("{payload}");
            HttpResponse::Ok().json(Success {
                request_id: request_id.to_string(),
                payload,
            })
        }
        Err(err) => {
            let err_string = err.to_string();
            tracing::error!("{err_string}");
            HttpResponse::InternalServerError().json(Fail {
                request_id: request_id.to_string(),
                error: err_string,
            })
        }
    }
}

#[tracing::instrument(name = "URL add", skip(redis_client, long_url))]
async fn add(redis_client: &Client, long_url: &str) -> Result<UrlResponsePayload, Error> {
    let key = &digest(long_url)[0..8];
    let short_url = format!("http://localhost/{key}");
    let mut payload = UrlResponsePayload {
        key: key.to_string(),
        long_url: long_url.to_string(),
        short_url,
    };

    let mut serialised =
        serde_json::to_string(&payload).map_err(|e| Error::Serialisation(e.to_string()))?;
    let mut conn = redis_client
        .get_connection()
        .map_err(|e| Error::RedisConnection(e.to_string()))?;

    match conn.get::<&str, String>(key) {
        Ok(current) => {
            let deserialised_current = serde_json::from_str::<UrlResponsePayload>(current.as_str())
                .map_err(|e| Error::Serialisation(e.to_string()))?;
            if payload.long_url != deserialised_current.long_url {
                let key = &digest(format!("{}a", payload.long_url))[0..8];
                payload.key = key.to_string();
                serialised = serde_json::to_string(&payload)
                    .map_err(|e| Error::Serialisation(e.to_string()))?;
                conn.set(key, serialised)
                    .map_err(|e| Error::RedisQuery(e.to_string()))?;
            }
        }
        Err(err) => match err.kind() {
            ErrorKind::TypeError => {
                conn.set(key, serialised)
                    .map_err(|e| Error::RedisQuery(e.to_string()))?;
            }
            _ => {
                Err(Error::RedisQuery(err.to_string()))?;
            }
        },
    }

    Ok(payload)
}
