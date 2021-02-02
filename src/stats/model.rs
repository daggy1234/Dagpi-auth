use serde::Deserialize;
use serde::Serialize;
use sqlx::{Pool, Postgres};
#[derive(Serialize)]
pub struct Resp<'a> {
    pub message: &'a str,
}

#[derive(Serialize)]
pub struct Stat {
    pub user_agent: String,
    pub route: String,
    pub api: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct StatForm {
    pub user_agent: String,
    pub route: String,
    pub api: String,
    pub token: String,
}

#[derive(Serialize)]
pub struct StatResp {
    pub total: usize,
    pub data: Vec<Stat>,
}

pub struct StatPool {
    pub pool: Pool<Postgres>,
}

impl Clone for StatPool {
    fn clone(&self) -> Self {
        let cp = self.pool.clone();
        StatPool { pool: cp }
    }
}
