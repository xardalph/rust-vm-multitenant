use super::super::super::model::Agent;
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
    State(db): State<sqlxPool<sqlx::Postgres>>,
    mut req: Request,
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
    let agent: Result<Agent, sqlx::Error> = sqlx::query_as::<_, Agent>(
        "
            SELECT id, name, token, id_company
            FROM agent
            WHERE token = $1
        ",
    )
    .bind(token)
    .fetch_one(&db)
    .await;

    // TODO if there is two agent with the same token there is a bug, should disable both by default.
    match agent {
        Ok(a) => {
            debug!("found agent {} in db.", a.name);
            req.extensions_mut().insert(a);
            return Ok(next.run(req).await);
        }
        Err(e) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // If agent exist, proceed to the next handler
}
