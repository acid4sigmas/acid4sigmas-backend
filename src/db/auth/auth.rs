use anyhow::Ok;
use sqlx::postgres::PgRow;
use sqlx::Row;
use sqlx::PgPool;
use anyhow::Result;

use crate::secrets::SECRETS;

pub struct Database {
    pub pool: PgPool
} 

#[derive(Debug, Clone)]
pub struct User {
    pub uid: i64,
    pub email: String,
    pub email_verified: bool,
    pub username: String,
    pub password_hash: String
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
            "CREATE TABLE IF NOT EXISTS auth_users (
                uid BIGINT PRIMARY KEY,
                email TEXT,
                email_verified BOOLEAN DEFAULT FALSE,
                username TEXT,
                password_hash TEXT
            )"
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert(&self, uid: i64, username: &str, password_hash: &str, email: &str) -> Result<()> {
        let mut txn = self.pool.begin().await?;

        sqlx::query("INSERT INTO auth_users (
            uid,
            email,
            username,
            password_hash
        ) VALUES ($1, $2, $3, $4)")
        .bind(uid)
        .bind(email)
        .bind(username)
        .bind(password_hash)
        .execute(&mut *txn)
        .await?;
        
        txn.commit().await?;

        Ok(())
    }

    pub async fn read_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM auth_users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        let user = match row {
            Some(row) => Some(parse_auth_user_record(row)?),
            None => None
        };

        Ok(user)
    }

    pub async fn read_by_uid(&self, uid: i64) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM auth_users WHERE uid = $1")
            .bind(uid)
            .fetch_optional(&self.pool)
            .await?;
        
        let user = match row {
            Some(row) => Some(parse_auth_user_record(row)?),
            None => None
        };

        Ok(user)
    }

    pub async fn read_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM auth_users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        let user = match row {
            Some(row) => Some(parse_auth_user_record(row)?),
            None => None
        };

        Ok(user)
    }

    pub async fn update_email_verification(&self, uid: i64, email_verification: bool) -> Result<()> {

        let mut txn = self.pool.begin().await?;

        sqlx::query("UPDATE auth_users SET email_verified = $1 WHERE uid = $2")
            .bind(email_verification)
            .bind(uid)
            .execute(&mut *txn)
            .await?;

        txn.commit().await?;

        Ok(())
    }

    pub async fn update_password(&self, uid: i64, password_hash: &str) -> Result<()> {
        let mut txn = self.pool.begin().await?;

        sqlx::query("UPDATE auth_users SET password_hash = $1 WHERE uid = $2")
            .bind(password_hash)
            .bind(uid)
            .execute(&mut *txn)
            .await?;

        txn.commit().await?;

        Ok(())
    }
}

fn parse_auth_user_record(row: PgRow) -> Result<User> {
    Ok(User {
        uid: row.try_get(0)?,
        email: row.try_get(1)?,
        email_verified: row.try_get(2)?,
        username: row.try_get(3)?,
        password_hash: row.try_get(4)?
    })
}