use std::collections::HashMap;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_login::tracing::debug;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use thiserror::Error;
use uuid::Uuid;
#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error")]
    Sqlx(#[from] sqlx::Error), // automatically implements From<sqlx::Error>

    #[error("empty argument")]
    EmptyArgument,
    #[error("trying to create an element already present")]
    AlreadyUsed,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        debug!("raised an error : {:#?}", &self);
        match self {
            AppError::Sqlx(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
            AppError::EmptyArgument => {
                (StatusCode::BAD_REQUEST, Json("Empty argument, check body")).into_response()
            }
            AppError::AlreadyUsed => (
                StatusCode::CONFLICT,
                Json("trying to create an element already present"),
            )
                .into_response(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub id_victoria: i32,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub id_company: Uuid,
}
// Here we've implemented `Debug` manually to avoid accidentally logging the
// password hash.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .field("id_company", &self.id_company)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub id_company: Uuid,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PubAgent {
    pub name: String,
    pub token: String,
}
// ex : {"metric":{"__name__":"evan-metric1","job":"curl","instance":"vmagent:8429"},"values":[100,300],"timestamps":[1763074402660,1763074402661]}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoriaMetric {
    pub metric: HashMap<String, String>,
    pub values: Vec<f64>,
    pub timestamps: Vec<i64>,
}
