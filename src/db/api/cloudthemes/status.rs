use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgPool, Row};
use anyhow::Result;

use crate::secrets::SECRETS;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudThemesStatus {
    pub enabled: bool
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
        let mut txn = self.pool.begin().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS cloudthemes_status (
                uid BIGINT PRIMARY KEY,
                enabled BOOLEAN DEFAULT FALSE
            )"
        )
        .execute(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(())
    }

    pub async fn update_status(&self, uid: i64, enabled: bool) -> Result<()> {
        let mut txn = self.pool.begin().await?;

        let result = sqlx::query("UPDATE cloudthemes_status SET enabled = $1 WHERE uid = $2")
            .bind(enabled)
            .bind(uid)
            .execute(&mut *txn)
            .await?;

        if result.rows_affected() == 0 {
            sqlx::query("INSERT INTO cloudthemes_status (uid, enabled) VALUES ($1, $2)")
                .bind(uid)
                .bind(enabled)
                .execute(&mut *txn)
                .await?;
        }

        txn.commit().await?;

        Ok(())
    }

    pub async fn read_status_by_uid(&self, uid: i64) -> Result<CloudThemesStatus> {


        match sqlx::query("SELECT * FROM cloudthemes_status WHERE uid = $1")
            .bind(uid)
            .fetch_one(&self.pool)
            .await 
        {
            Ok(row) => Ok(CloudThemesStatus {
                enabled: row.try_get(1)?
            }),
            Err(sqlx::Error::RowNotFound) => {
                sqlx::query("INSERT INTO cloudthemes_status (uid, enabled) VALUES ($1, $2)")
                .bind(uid)
                .bind(false)  
                .execute(&self.pool)
                .await?;

                Ok(CloudThemesStatus {
                    enabled: false
                })
            }
            Err(e) => Err(e.into())
        }
        
      

    }

}
