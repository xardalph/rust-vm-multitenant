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
        .route("/agent", get(self::agent::get))
        .route("/agent", post(self::agent::post))
}
mod agent {
    use super::*;
    use axum::Json;
    #[derive(serde::Serialize)]
    pub struct Agent {
        id: i64,
        name: String,
    }

    pub async fn get(
        // access the state via the `State` extractor
        // extracting a state of the wrong type results in a compile error
        State(db): State<sqlxPool<sqlx::Any>>,
    ) -> Result<(http::StatusCode, axum::Json<Agent>), http::StatusCode> {
        return Ok((
            StatusCode::OK,
            Json(Agent {
                id: 60,
                name: "agent1".to_string(),
            }),
        ));
    }
    pub async fn post(
        // access the state via the `State` extractor
        // extracting a state of the wrong type results in a compile error
        State(state): State<sqlxPool<sqlx::Any>>,
    ) -> Result<(http::StatusCode, axum::Json<Agent>), http::StatusCode> {
        return Ok((
            StatusCode::OK,
            Json(Agent {
                id: 60,
                name: "agent1".to_string(),
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
