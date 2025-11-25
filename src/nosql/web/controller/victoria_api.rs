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
        Extension, debug_handler,
        extract::{self, State},
    };
    use axum_login::tracing::debug;
    use serde::Serialize;
    use uuid::Uuid;

    use crate::nosql::model::AppError;

    use super::super::super::super::{
        model::{Agent, VictoriaMetric},
        users::AuthSession,
    };
    use super::*;
    pub async fn get_victoria_tenant_from_agent(
        db: sqlxPool<sqlx::Postgres>,
        agent: &Agent,
    ) -> Result<i32, AppError> {
        let agent_victoria_id: (i32,) = sqlx::query_as(
            "
                SELECT id_victoria
                FROM company
                WHERE id = $1 LIMIT 1
            ",
        )
        .bind(agent.id_company)
        .fetch_one(&db)
        .await?;
        return Ok(agent_victoria_id.0);
    }

    pub async fn insert(
        Extension(agent): Extension<Agent>,
        State(db): State<sqlxPool<sqlx::Postgres>>,
        State(client): State<reqwest::Client>,
        extract::Json(mut payload): extract::Json<VictoriaMetric>,
    ) -> Result<http::StatusCode, AppError> {
        let url = format!(
            "http://localhost:8427/insert/{}/prometheus/api/v1/import",
            get_victoria_tenant_from_agent(db, &agent).await?,
        );
        debug!(
            "trying to request url {} with body {}",
            url,
            serde_json::to_string(&payload).unwrap()
        );
        payload.metric.insert("job".to_string(), agent.name.clone());

        let req = client
            .post(url)
            .basic_auth("foo", Some("bar"))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&payload).unwrap());
        let res = req
            .send()
            .await
            .or_else(|e| -> Result<reqwest::Response, _> {
                println!("err : {:?}", &e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            });
        debug!("sent a post request, result : {:?}", res);
        return Ok(StatusCode::OK);
    }
    pub async fn select(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        "html message from page 2".into_response()
    }
}
