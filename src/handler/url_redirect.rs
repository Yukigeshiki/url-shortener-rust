use actix_web::{web, HttpResponse};
use redis::{Client, Commands};
use tracing_actix_web::RequestId;

use crate::handler::{Error, Fail, UrlResponsePayload};

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "URL redirect route",
    skip(redis_client),
    fields(
        param = % path_param
    )
)]
pub async fn url_redirect(
    request_id: RequestId,
    path_param: web::Path<String>,
    redis_client: web::Data<Client>,
) -> HttpResponse {
    match get(&redis_client, path_param.into_inner().as_str()).await {
        Ok(payload) => {
            tracing::info!("{payload}");
            HttpResponse::Found()
                .insert_header(("Location", payload.long_url))
                .finish()
        }
        Err(err) => {
            let err_string = err.to_string();
            tracing::error!("{err_string}");
            let fail_response = Fail {
                request_id: request_id.to_string(),
                error: err_string,
            };
            match err {
                Error::NotFound(_) => HttpResponse::NotFound().json(fail_response),
                _ => HttpResponse::InternalServerError().json(fail_response),
            }
        }
    }
}

async fn get(redis_client: &Client, key: &str) -> Result<UrlResponsePayload, Error> {
    let mut conn = redis_client
        .get_connection()
        .map_err(|e| Error::RedisConnection(e.to_string()))?;
    match conn.exists::<&str, i8>(key) {
        Ok(val) => {
            if val > 0 {
                let payload = conn
                    .get::<&str, String>(key)
                    .map_err(|e| Error::RedisQuery(e.to_string()))?;
                serde_json::from_str(payload.as_str())
                    .map_err(|e| Error::Serialisation(e.to_string()))
            } else {
                Err(Error::NotFound(key.to_string()))?
            }
        }
        Err(err) => Err(Error::RedisQuery(err.to_string()))?,
    }
}
