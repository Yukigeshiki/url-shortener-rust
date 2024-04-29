use actix_web::{web, HttpResponse};
use redis::{Client, Commands, ErrorKind};
use tracing_actix_web::RequestId;

use crate::handler::{Error, Fail, QueryResult, UrlResponsePayload};

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

#[tracing::instrument(name = "URL get", skip(redis_client, key))]
async fn get(redis_client: &Client, key: &str) -> QueryResult<UrlResponsePayload> {
    let mut conn = redis_client
        .get_connection()
        .map_err(|e| Error::RedisConnection(e.to_string()))?;
    match conn.get::<&str, String>(key) {
        Ok(payload) => {
            serde_json::from_str(payload.as_str()).map_err(|e| Error::Serialisation(e.to_string()))
        }
        Err(err) => match err.kind() {
            ErrorKind::TypeError => Err(Error::NotFound(key.to_string()))?,
            _ => Err(Error::RedisQuery(err.to_string()))?,
        },
    }
}
