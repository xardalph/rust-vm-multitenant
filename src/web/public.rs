use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use axum_messages::{Message, Messages};
use serde::Deserialize;
use sqlx::Pool as sqlxPool;
use sqlx::{AnyPool, SqlitePool};

use crate::users::{AuthSession, Credentials};

pub fn router() -> Router<sqlxPool<sqlx::Any>> {
    Router::new()
        .route("/test", get(self::get::test))
        .route("/test2", get(self::get::test2))
}

mod get {
    use super::*;
    use crate::users::AuthSession;

    pub async fn test() -> impl IntoResponse {
        "html message".into_response()
    }
    pub async fn test2(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        "html message from page 2".into_response()
    }
}
