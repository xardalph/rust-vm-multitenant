use super::super::super::users::{AuthSession, Credentials};

use super::super::super::web::{App, extractor::current_user::*};
use askama::Template;
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
        .route("/agent/{name}", get(self::agent::get_one))
        .route("/select", post(self::victoria_api::select))
}
mod victoria_api {
    use axum::extract::{self, Request};
    use axum_login::tracing::{debug, error};

    use crate::nosql::{model::VictoriaMetric, web::app::VictoriaEndpoint};

    use super::*;
    use bytes::Bytes;
    pub async fn select(
        user: CurrentUser,
        State(client): State<reqwest::Client>,
        State(vm_url): State<VictoriaEndpoint>,
        body: Bytes,
    ) -> Result<(http::StatusCode, Bytes), http::StatusCode> {
        let url = format!(
            "{}/select/{}/prometheus/api/v1/export",
            vm_url.url, user.id_company
        );
        error!("trying request {url}");
        let req = client
            .post(url)
            .basic_auth("foo", Some("bar"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body);
        error!("body : {:#?}", &req);

        let res = req
            .send()
            .await
            .or_else(|e| -> Result<reqwest::Response, _> {
                error!("http error on VM select : {:?}", &e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            })?;
        error!("VM response : {:#?}", &res);

        return Ok((res.status(), res.bytes().await.unwrap()));
        //return Err(StatusCode::NOT_IMPLEMENTED);
    }
}
mod agent {
    use super::*;
    use axum::{Json, extract::Path};
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
