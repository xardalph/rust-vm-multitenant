use super::super::super::model::User;
use super::super::super::users::{self};
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use uuid::Uuid;

#[derive(Debug)]
pub struct CurrentUser {
    pub id: Uuid,
    pub id_company: Uuid,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let clone = parts
            .extensions
            .get::<axum_login::AuthSession<users::Backend>>()
            .cloned();
        let user = clone.unwrap().user.unwrap();

        Ok(Self {
            id: user.id,
            id_company: user.id_company,
        })
    }
}
