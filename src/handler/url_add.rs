use std::fmt::{Display, Formatter};

use actix_web::{web, HttpResponse};
use redis::{Client, Commands};
use sha256::digest;
use tracing_actix_web::RequestId;

use crate::handler::{Error, Fail, Success};
use crate::impl_json_display;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct LongUrlJsonData {
    long_url: String,
}

impl_json_display!(LongUrlJsonData);

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponsePayload {
    pub key: String,
    pub long_url: String,
    pub short_url: String,
}

impl_json_display!(ResponsePayload);

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
    json_data: web::Json<LongUrlJsonData>,
    redis_client: web::Data<Client>,
) -> HttpResponse {
    match add(&redis_client, json_data.0.long_url).await {
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
async fn add(redis_client: &Client, long_url: String) -> Result<ResponsePayload, Error> {
    let key = &digest(&long_url)[0..6];
    let short_url = format!("http://localhost/{key}");
    let mut payload = ResponsePayload {
        key: key.to_string(),
        long_url,
        short_url,
    };

    let serialised =
        serde_json::to_string(&payload).map_err(|e| Error::Serialisation(e.to_string()))?;
    let mut conn = redis_client
        .get_connection()
        .map_err(|e| Error::RedisConnection(e.to_string()))?;
    match conn.exists::<&str, i8>(key) {
        Ok(val) => {
            if val > 0 {
                let current = conn
                    .get::<&str, String>(key)
                    .map_err(|e| Error::RedisQuery(e.to_string()))?;
                let deserialised_current =
                    serde_json::from_str::<ResponsePayload>(current.as_str())
                        .map_err(|e| Error::Serialisation(e.to_string()))?;
                if payload.long_url != deserialised_current.long_url {
                    let key = &digest(format!("{}a", payload.long_url))[0..6];
                    payload.key = key.to_string();
                    let serialised = serde_json::to_string(&payload)
                        .map_err(|e| Error::Serialisation(e.to_string()))?;
                    conn.set(key, serialised)
                        .map_err(|e| Error::RedisQuery(e.to_string()))?;
                }
            } else {
                conn.set(key, serialised)
                    .map_err(|e| Error::RedisQuery(e.to_string()))?;
            }
        }
        Err(err) => {
            Err(Error::RedisQuery(err.to_string()))?;
        }
    }

    Ok(payload)
}
