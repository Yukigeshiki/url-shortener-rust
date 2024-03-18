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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ResponsePayload {
    key: String,
    long_url: String,
    short_url: String,
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
        Err(err) => HttpResponse::InternalServerError().json(Fail {
            request_id: request_id.to_string(),
            error: err.to_string(),
        }),
    }
}

#[tracing::instrument(name = "URL add", skip(redis_client, long_url))]
async fn add(redis_client: &Client, long_url: String) -> Result<ResponsePayload, Error> {
    let key = &digest(&long_url)[0..6];
    let short_url = format!("http://localhost/{key}");
    let payload = ResponsePayload {
        key: key.to_string(),
        long_url,
        short_url,
    };

    let serialised =
        serde_json::to_string(&payload).map_err(|e| Error::Serialisation(e.to_string()))?;
    let mut conn = redis_client
        .get_connection()
        .map_err(|e| Error::RedisConnection(e.to_string()))?;
    if conn.exists::<&str, String>(key).is_err() {
        conn.set(key, serialised)
            .map_err(|e| Error::RedisQuery(e.to_string()))?;
    }

    Ok(payload)
}