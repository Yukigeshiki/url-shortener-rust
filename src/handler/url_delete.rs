use actix_web::{web, HttpResponse};
use redis::{Client, Commands};
use tracing_actix_web::RequestId;

use crate::handler::{Error, Fail, QueryResult, Success};

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "URL redirect route",
    skip(redis_client, request_id, path_param),
    fields(
        param = % path_param
    )
)]
pub async fn url_delete(
    request_id: RequestId,
    path_param: web::Path<String>,
    redis_client: web::Data<Client>,
) -> HttpResponse {
    match delete(&redis_client, path_param.into_inner().as_str()).await {
        Ok(()) => {
            let msg = "URL deleted successfully!";
            tracing::info!(msg);
            HttpResponse::Ok().json(Success {
                request_id: request_id.to_string(),
                payload: msg,
            })
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

async fn delete(redis_client: &Client, key: &str) -> QueryResult<()> {
    let mut conn = redis_client
        .get_connection()
        .map_err(|e| Error::RedisConnection(e.to_string()))?;
    match conn.del::<&str, i8>(key) {
        Ok(val) => {
            if val > 0 {
                Ok(())
            } else {
                Err(Error::NotFound(key.to_string()))?
            }
        }
        Err(err) => Err(Error::RedisQuery(err.to_string()))?,
    }
}
