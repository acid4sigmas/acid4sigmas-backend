
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgPool, Row};
use anyhow::Result;

use crate::secrets::SECRETS;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub uid: i64,
    pub email: String,
    pub email_verified: String,
    pub owner: bool,
    pub username: String
}

pub struct Database {
    pub pool: PgPool
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

    pub async fn create_table(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                uid BIGINT PRIMARY KEY,
                email TEXT,
                owner BOOLEAN DEFAULT FALSE,
                email_verified BOOLEAN DEFAULT FALSE,
                username TEXT
            )"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert(&self, uid: i64, username: &str, email: &str) -> Result<()> {
        let mut txn = self.pool.begin().await?;

        sqlx::query("INSERT INTO users (
                uid,
                email,
                username
            ) VALUES ($1, $2, $3)"
        )
        .bind(uid)
        .bind(email)
        .bind(username)
        .execute(&mut *txn)
        .await?;

        Ok(())
    }

    pub async fn read_by_uid(&self, uid: i64) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE uid = $1")
            .bind(uid)
            .fetch_optional(&self.pool)
            .await?;
        
        let user = match row {
            Some(row) => Some(parse_auth_user_record(row)?),
            None => None
        };

        Ok(user)
    }


 
}

fn parse_auth_user_record(row: PgRow) -> Result<User> {
    Ok(User {
        uid: row.try_get(0)?,
        email: row.try_get(1)?,
        owner: row.try_get(2)?,
        email_verified: row.try_get(3)?,
        username: row.try_get(4)?,
    })
}