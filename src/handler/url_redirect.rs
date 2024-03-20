use actix_web::http::StatusCode;
use actix_web::web::Redirect;
use actix_web::{web, HttpResponse};
use redis::{Client, Commands};
use tracing_actix_web::RequestId;

use crate::handler::{Error, Fail, ResponsePayload};

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "URL redirect route",
    skip(redis_client),
    fields(
        param = % path_param
    )
)]
pub async fn url_redirect(
    path_param: web::Path<String>,
    redis_client: web::Data<Client>,
) -> Redirect {
    match get(&redis_client, path_param.into_inner().as_str()).await {
        Ok(payload) => {
            tracing::info!("{payload}");
            Redirect::to(payload.long_url)
                .permanent()
                .using_status_code(StatusCode::FOUND)
        }
        Err(err) => {
            let path = format!("/redirect-error/{err}");
            Redirect::to(path).permanent()
        }
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(name = "URL redirect error route", skip(request_id))]
pub async fn url_redirect_error(
    request_id: RequestId,
    path_param: web::Path<String>,
) -> HttpResponse {
    HttpResponse::InternalServerError().json(Fail {
        request_id: request_id.to_string(),
        error: path_param.into_inner(),
    })
}

async fn get(redis_client: &Client, key: &str) -> Result<ResponsePayload, Error> {
    let mut conn = redis_client
        .get_connection()
        .map_err(|e| Error::RedisConnection(e.to_string()))?;
    let payload = conn
        .get::<&str, String>(key)
        .map_err(|e| Error::RedisQuery(e.to_string()))?;

    serde_json::from_str(payload.as_str()).map_err(|e| Error::Serialisation(e.to_string()))
}
