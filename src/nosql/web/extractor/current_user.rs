use crate::nosql::model::AppError;

use super::super::super::model::User;
use super::super::super::users::{self};
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use sqlx::Pool as sqlxPool;
use uuid::Uuid;
#[derive(Debug)]
pub struct CurrentUser {
    pub id: Uuid,
    pub id_company: Uuid,
}
impl CurrentUser {
    pub async fn id_victoria(&self, db: sqlxPool<sqlx::Postgres>) -> Result<i32, AppError> {
        let id_victoria: (i32,) = sqlx::query_as(
            "
                SELECT id_victoria
                FROM company
                WHERE id = $1 LIMIT 1
            ",
        )
        .bind(self.id_company)
        .fetch_one(&db)
        .await?;
        return Ok(id_victoria.0);
    }
}
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_session = parts
            .extensions
            .get::<axum_login::AuthSession<users::Backend>>()
            .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))?;
        
        let user = auth_session.user.as_ref()
            .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))?;

        Ok(Self {
            id: user.id,
            id_company: user.id_company,
        })
    }
}
