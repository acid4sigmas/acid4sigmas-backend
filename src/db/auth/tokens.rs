// for people who do not understand the upcoming code, THIS DOES NOT STORE TOKENS. it certainly only stores some token details which are required to invalidate a token

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use sqlx::postgres::PgRow;
use sqlx::Row;
use sqlx::PgPool;
use anyhow::Result;

use crate::secrets::SECRETS;

pub struct Database {
    pub pool: PgPool
} 

#[derive(Debug)]
pub struct AuthTokens {
    pub uid: i64,
    pub jti: String,
    pub expires_at: DateTime<Utc>
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
            "CREATE TABLE IF NOT EXISTS auth_tokens (
                jti TEXT PRIMARY KEY,
                uid BIGINT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn insert(&self, user_id: i64, jti: &str, expires_at: DateTime<Utc>) -> Result<()> {
        let mut txn = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO auth_tokens (
                jti,
                uid,
                expires_at
            ) VALUES ($1, $2, $3)"
        )
        .bind(jti)
        .bind(user_id)
        .bind(expires_at)
        .execute(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(())
    }

    pub async fn read_by_uid(&self, user_id: i64) -> Result<Vec<String>> {
        let results = sqlx::query(
            "SELECT * FROM auth_tokens WHERE uid = $1 AND expires_at > NOW()"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results.into_iter().map(|record| parse_auth_tokens(record).unwrap().jti).collect())
    }

    pub async fn delete_by_uid(&self, user_id: i64) -> Result<()> {

        let mut txn = self.pool.begin().await?;

        let result = sqlx::query("DELETE FROM auth_tokens WHERE uid = $1")
            .bind(user_id)
            .execute(&mut *txn)
            .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                println!("Deleted all rows with uid = {}", user_id);
                Ok(())
            }
            Err(e) => {
                    // Rollback the transaction in case of an error
                txn.rollback().await?;
                println!("Failed to delete rows with uid = {}: {}", user_id, e);
                Err(e.into())
            }
        }


    }
}

fn parse_auth_tokens(row: PgRow) -> Result<AuthTokens> {
    Ok(AuthTokens {
        jti: row.try_get(0)?,
        uid: row.try_get(1)?,
        expires_at: row.try_get(2)?
    })
}