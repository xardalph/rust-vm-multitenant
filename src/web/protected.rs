use crate::web::extractor::current_user::*;
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use axum_messages::{Message, Messages};

use crate::users::AuthSession;
use sqlx::Pool as sqlxPool;

#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    messages: Vec<Message>,
    username: &'a str,
}

pub fn router() -> Router<sqlxPool<sqlx::Any>> {
    Router::new()
        .route("/", get(self::get::protected))
        .route("/agent", get(self::agent::get).post(self::agent::post))
        .route("/agent/{name}", get(self::agent::get_one))
}
mod agent {
    use super::*;
    use axum::{extract::Path, Json};
    use futures_util::TryStreamExt;
    use sqlx::Row;
    #[derive(serde::Serialize)]
    pub struct Agent {
        name: String,
        token: String,
    }
    pub async fn get_one(
        Path(key): Path<String>,
        user: CurrentUser,
        State(db): State<sqlxPool<sqlx::Any>>,
    ) -> Result<(http::StatusCode, axum::Json<Vec<Agent>>), http::StatusCode> {
        // TODO : implement a get for only one agent by name, searching only for agent in the company of the logger user.
        // If no agent found (or agent for another company) -> 404
        return Err(StatusCode::NOT_IMPLEMENTED);
    }
    pub async fn get(
        user: CurrentUser,
        State(db): State<sqlxPool<sqlx::Any>>,
    ) -> Result<(http::StatusCode, axum::Json<Vec<Agent>>), http::StatusCode> {
        let mut rows = sqlx::query("SELECT * FROM agent WHERE id_company = $1")
            .bind(user.id_company)
            .fetch(&db);
        let mut agents = vec![];
        while let Some(row) = rows.try_next().await.unwrap() {
            // map the row into a user-defined domain type

            let name = row.try_get::<&str, &str>("name").unwrap().to_string();
            let token = row.try_get::<&str, &str>("token").unwrap().to_string();
            agents.push(Agent { name, token });
        }

        return Ok((StatusCode::OK, Json(agents)));
    }
    pub async fn post(
        // access the state via the `State` extractor
        // extracting a state of the wrong type results in a compile error
        State(state): State<sqlxPool<sqlx::Any>>,
    ) -> Result<(http::StatusCode, axum::Json<Agent>), http::StatusCode> {
        return Ok((
            StatusCode::OK,
            Json(Agent {
                name: "agent1".to_string(),
                token: "tken".to_string(),
            }),
        ));
    }
}
mod get {

    use super::*;

    pub async fn protected(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => Html(
                ProtectedTemplate {
                    messages: messages.into_iter().collect(),
                    username: &user.username,
                }
                .render()
                .unwrap(),
            )
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
