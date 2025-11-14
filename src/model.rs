use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub id_company: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: i64,
    pub name: String,
    pub token: String,
    pub id_company: i64,
}
