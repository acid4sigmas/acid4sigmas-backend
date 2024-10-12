pub mod api;
pub mod auth;
use crate::secrets::SECRETS;
use anyhow::Result;
use once_cell::sync::Lazy;
use sqlx::{Error, PgPool};

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let url = format!(
            "postgresql://postgres:{}@localhost:{}/{}",
            SECRETS.get("DB_PW").unwrap(),
            SECRETS.get("DB_PORT").unwrap(),
            SECRETS.get("DB_NAME").unwrap()
        );
        let pool = sqlx::postgres::PgPool::connect(&url).await?;
        Ok(Self { pool })
    }

    pub async fn get_pool() -> Result<PgPool> {
        Database::new().await.map(|db| db.pool)
    }
}
