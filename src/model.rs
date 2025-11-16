use std::collections::HashMap;

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
// ex : {"metric":{"__name__":"evan-metric1","job":"curl","instance":"vmagent:8429"},"values":[100,300],"timestamps":[1763074402660,1763074402661]}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoriaInsert {
    pub metric: HashMap<String, String>,
    pub values: Vec<f64>,
    pub timestamps: Vec<i64>,
}
