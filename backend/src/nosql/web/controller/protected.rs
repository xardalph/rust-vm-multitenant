use super::super::super::users::{AuthSession, Credentials};

use super::super::super::web::{App, extractor::current_user::*};
use askama::Template;
use axum::{
    Json,
    extract::{self, Path},
};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use axum_messages::{Message, Messages};
use sqlx::Pool as sqlxPool;

#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    messages: Vec<Message>,
    username: &'a str,
}

pub fn router() -> Router<App> {
    Router::new()
        .route("/", get(self::get::protected))
        .route("/agent", get(self::agent::get).post(self::agent::post))
        .route(
            "/agent/{name}",
            get(self::agent::get_one).delete(self::agent::delete),
        )
        // this redirect to Victoria metric api
        .route(
            "/vm/{*path}",
            get(self::victoria_api::get).post(self::victoria_api::post),
        )
}
mod victoria_api {
    use axum::extract::{self, Request};
    use axum_login::tracing::{debug, error, info};
    use docker_api::models::TaskStatusInlineItemContainerStatusInlineItem;
    use http::HeaderMap;
    use reqwest::RequestBuilder;

    use crate::nosql::{
        model::{Agent, VictoriaMetric},
        web::app::VictoriaEndpoint,
    };

    use super::*;
    use bytes::Bytes;

    pub async fn get(
        user: CurrentUser,
        State(client): State<reqwest::Client>,
        State(vm_url): State<VictoriaEndpoint>,
        State(db): State<sqlxPool<sqlx::Postgres>>,
        Path(path): Path<String>,
        headers: HeaderMap,
    ) -> Result<(http::StatusCode, Bytes), http::StatusCode> {
        let url = format!(
            "{}/select/{}/prometheus/api/v1/{}",
            vm_url.url,
            user.id_victoria(db).await.unwrap(),
            path,
        );
        let mut req = client.get(url);
        for (k, v) in headers {
            if k.is_none() {
                continue;
            }
            req = req.header(k.unwrap(), v);
        }

        let res = req
            .send()
            .await
            .or_else(|e| -> Result<reqwest::Response, _> {
                error!("http error on VM query : {:?}", &e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            })?;
        error!("VM response : {:#?}", &res);

        return Ok((res.status(), res.bytes().await.unwrap()));
    }
    pub async fn post(
        user: CurrentUser,
        State(client): State<reqwest::Client>,
        State(vm_url): State<VictoriaEndpoint>,
        State(db): State<sqlxPool<sqlx::Postgres>>,
        Path(path): Path<String>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Result<(http::StatusCode, Bytes), http::StatusCode> {
        let url = format!(
            "{}/select/{}/prometheus/api/v1/{}",
            vm_url.url,
            user.id_victoria(db).await.unwrap(),
            path,
        );
        let mut req = client.post(url).body(body);
        for (k, v) in headers {
            if k.is_none() {
                continue;
            }
            req = req.header(k.unwrap(), v);
        }

        let res = req
            .send()
            .await
            .or_else(|e| -> Result<reqwest::Response, _> {
                error!("http error on VM query : {:?}", &e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            })?;
        error!("VM response : {:#?}", &res);

        return Ok((res.status(), res.bytes().await.unwrap()));
    }
}
mod agent {
    use crate::nosql::model::{Agent, AppError, PubAgent};

    use super::*;

    use axum_login::tracing::info;
    use futures_util::TryStreamExt;
    use sqlx::Row;
    use uuid::Uuid;

    pub async fn get_one(
        Path(agent_name): Path<String>,
        user: CurrentUser,
        State(db): State<sqlxPool<sqlx::Postgres>>,
    ) -> Result<(http::StatusCode, axum::Json<Option<Agent>>), AppError> {
        // If no agent found (or agent for another company) -> 404
        let agent: Option<Agent> = sqlx::query_as::<_, Agent>(
            "
                SELECT *
                FROM agent
                WHERE id_company = $1 and name = $2
            ",
        )
        .bind(user.id_company)
        .bind(agent_name.to_string())
        .fetch_optional(&db)
        .await?;
        match agent {
            Some(a) => {
                return Ok((StatusCode::OK, Json(Some(a))));
            }
            None => return Ok((StatusCode::NOT_FOUND, Json(None))),
        }
    }
    pub async fn delete(
        Path(agent_id): Path<Uuid>,
        user: CurrentUser,
        State(db): State<sqlxPool<sqlx::Postgres>>,
    ) -> Result<(http::StatusCode), AppError> {
        // If no agent found (or agent for another company) -> 404
        let agent: Option<Agent> = sqlx::query_as::<_, Agent>(
            "
                SELECT *
                FROM agent
                WHERE id_company = $1 and id = $2
            ",
        )
        .bind(user.id_company)
        .bind(agent_id)
        .fetch_optional(&db)
        .await?;

        match agent {
            None => return Ok(StatusCode::NOT_FOUND),
            Some(a) => {
                match sqlx::query(
                    "
                        delete from agent WHERE id_company = $1 and id = $2
                    ",
                )
                .bind(user.id_company)
                .bind(agent_id)
                .execute(&db)
                .await
                {
                    Ok(_) => {
                        return Ok(StatusCode::OK);
                    }
                    Err(e) => {
                        if let Some(db_err) = e.as_database_error() {
                            if let Some(code) = db_err.code() {
                                // 23505 = unique_violation
                                if code == "23505" {
                                    return Err(AppError::AlreadyUsed);
                                }
                            }
                        }
                        // Other SQL or unexpected error
                        Err(e.into())
                    }
                }
            }
        }
    }

    pub async fn get(
        user: CurrentUser,
        State(db): State<sqlxPool<sqlx::Postgres>>,
    ) -> Result<(http::StatusCode, axum::Json<Vec<Agent>>), AppError> {
        let agents: Vec<Agent> = sqlx::query_as::<_, Agent>(
            "
                SELECT *
                FROM agent
                WHERE id_company = $1
            ",
        )
        .bind(user.id_company)
        .fetch_all(&db)
        .await?;
        return Ok((StatusCode::OK, Json(agents)));
    }

    pub async fn post(
        user: CurrentUser,
        State(db): State<sqlxPool<sqlx::Postgres>>,
        extract::Json(new_agent): extract::Json<PubAgent>,
    ) -> Result<impl IntoResponse, AppError> {
        if new_agent.name == "" {
            return Err(AppError::EmptyArgument);
        }
        let result = sqlx::query(
            "
                INSERT INTO agent(name, token, id_company)
                values($1,$2,$3)
            ",
        )
        .bind(new_agent.name)
        .bind(new_agent.token)
        .bind(user.id_company)
        .execute(&db)
        .await;
        match result {
            Ok(_) => Ok((StatusCode::CREATED, Json("created agent."))),
            Err(e) => {
                if let Some(db_err) = e.as_database_error() {
                    if let Some(code) = db_err.code() {
                        // 23505 = unique_violation
                        if code == "23505" {
                            return Err(AppError::AlreadyUsed);
                        }
                    }
                }
                // Other SQL or unexpected error
                Err(e.into())
            }
        }
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
