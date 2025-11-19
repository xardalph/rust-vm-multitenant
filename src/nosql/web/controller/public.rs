use askama::Template;
use axum::{
    Form, Router,
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use axum_messages::{Message, Messages};
use serde::Deserialize;
use sqlx::Pool as sqlxPool;
use sqlx::{AnyPool, SqlitePool};

use super::super::super::{
    users::{AuthSession, Credentials},
    web::App,
};

pub fn router() -> Router<App> {
    Router::new()
        .route("/test", get(self::get::test))
        .route("/test2", get(self::get::test2))
}

mod get {
    use super::super::super::super::users::AuthSession;
    use super::*;

    pub async fn test() -> impl IntoResponse {
        "html message".into_response()
    }
    pub async fn test2(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        "html message from page 2".into_response()
    }
}
