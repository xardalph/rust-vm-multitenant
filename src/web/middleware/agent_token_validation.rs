use crate::model::Agent;
use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use axum_login::tracing::debug;
use futures_util::TryStreamExt;
use sqlx::Pool as sqlxPool;
use sqlx::Row;

pub async fn check_api_token_against_agent_table(
    State(db): State<sqlxPool<sqlx::Any>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    //retrieve the Authorization http header, and check it start with Baerer
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    debug!("Token : '{:?}'", auth_header);

    let auth_header = if let Some(auth_header) = auth_header
        && auth_header.starts_with("Bearer ")
    {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let token = &auth_header[7..].to_string();
    // check the token against the client
    let mut rows = sqlx::query("SELECT name, token FROM agent WHERE token = $1")
        .bind(token)
        .fetch(&db);

    match rows.try_next().await.unwrap() {
        Some(row) => {
            let id = row
                .try_get::<&str, &str>("id")
                .unwrap()
                .parse::<i64>()
                .unwrap();
            let name = row.try_get::<&str, &str>("name").unwrap().to_string();
            let token = row.try_get::<&str, &str>("token").unwrap().to_string();
            let id_company = row.unwrap().parse::<i64>().unwrap();
            let a = Agent {
                id,
                name,
                token,
                id_company,
            };
            return Ok(next.run(req).await);
        }
        None => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    // TODO if there is two agent with the same token there is a bug, disabling both by default.

    // If the API key matches, proceed to the next handler
}
