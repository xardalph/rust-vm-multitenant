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
    Router::new().route("/insert", post(self::post::insert))
    //.route("/select", post(self::post::insert))
}

mod post {
    use axum::{
        Extension,
        extract::{self, State},
    };
    use axum_login::tracing::debug;
    use serde::Serialize;

    use super::super::super::super::{
        model::{Agent, VictoriaMetric},
        users::AuthSession,
    };
    use super::*;

    pub async fn insert(
        Extension(agent): Extension<Agent>,
        State(db): State<sqlxPool<sqlx::Any>>,
        State(client): State<reqwest::Client>,
        extract::Json(payload): extract::Json<VictoriaMetric>,
    ) -> impl IntoResponse {
        let url = format!(
            "http://localhost:8427/insert/0/prometheus/api/v1/import",
            //agent.id_company
        );
        debug!(
            "trying to request url {} with body {}",
            url,
            serde_json::to_string(&payload).unwrap()
        );
        let res = client
            .post(url)
            .basic_auth("foo", Some("bar"))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&payload).unwrap())
            .send()
            .await
            .unwrap();
        debug!("sent a post request, result : {:?}", res);
        format!("got a json payload : {:?}", payload).into_response()
    }
    pub async fn select(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        "html message from page 2".into_response()
    }
}
