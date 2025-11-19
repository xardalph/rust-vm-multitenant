use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

use super::super::super::users::{self, User};

#[derive(Debug)]
pub struct CurrentUser {
    pub id: i64,
    pub id_company: i64,
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
