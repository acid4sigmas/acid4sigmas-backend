use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgPool, Row};
use anyhow::Result;

use crate::secrets::SECRETS;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTheme {
    pub uid: i64,
    pub theme: Theme
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    primary_color_text: String,
    primary_color: String,
    secondary_color: String,
    background_color_primary: String,
    background_color_secondary: String,
    background_color_tertiary: String,
    primary_grey: String,
    secondary_grey: String,
    font_size: String,
    transparency: bool,
    transparency_value: f64,
    transparency_blur: String,
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
            "CREATE TABLE IF NOT EXISTS cloudthemes (
                uid BIGINT PRIMARY KEY,
                primary_color_text TEXT,
                primary_color TEXT,
                secondary_color TEXT,
                background_color_primary TEXT,
                background_color_secondary TEXT,
                background_color_tertiary TEXT,
                primary_grey TEXT,
                secondary_grey TEXT,
                font_size TEXT,
                transparency BOOLEAN DEFAULT TRUE,
                transparency_value FLOAT NOT NULL,
                transparency_blur TEXT
            )"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn read_by_uid(&self, uid: i64) -> Result<Option<CloudTheme>> {
        let row = sqlx::query("SELECT * FROM cloudthemes WHERE uid = $1")
            .bind(uid)
            .fetch_optional(&self.pool)
            .await?;
        
        let user = match row {
            Some(row) => Some(parse_cloudtheme_record(row)?),
            None => None
        };

        Ok(user)
    }


    pub async fn insert(&self, uid: i64, theme: Theme) -> Result<()> {
        let mut txn = self.pool.begin().await?;

        sqlx::query("INSERT INTO cloudthemes (
            uid,
            primary_color_text,
            primary_color,
            secondary_color,
            background_color_primary,
            background_color_secondary,
            background_color_tertiary,
            primary_grey,
            secondary_grey,
            font_size,
            transparency,
            transparency_value,
            transparency_blur
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT (uid) DO UPDATE SET
            primary_color_text = EXCLUDED.primary_color_text,
            primary_color = EXCLUDED.primary_color,
            secondary_color = EXCLUDED.secondary_color,
            background_color_primary = EXCLUDED.background_color_primary,
            background_color_secondary = EXCLUDED.background_color_secondary,
            background_color_tertiary = EXCLUDED.background_color_tertiary,
            primary_grey = EXCLUDED.primary_grey,
            secondary_grey = EXCLUDED.secondary_grey,
            font_size = EXCLUDED.font_size,
            transparency = EXCLUDED.transparency,
            transparency_value = EXCLUDED.transparency_value,
            transparency_blur = EXCLUDED.transparency_blur
        "
        )
        .bind(uid)
        .bind(&theme.primary_color_text)
        .bind(&theme.primary_color)
        .bind(&theme.secondary_color)
        .bind(&theme.background_color_primary)
        .bind(&theme.background_color_secondary)
        .bind(&theme.background_color_tertiary)
        .bind(&theme.primary_grey)
        .bind(&theme.secondary_grey)
        .bind(&theme.font_size)
        .bind(theme.transparency)
        .bind(theme.transparency_value)
        .bind(&theme.transparency_blur)
        .execute(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(())
    }


}

fn parse_cloudtheme_record(row: PgRow) -> Result<CloudTheme, sqlx::Error> {
    Ok(CloudTheme {
        uid: row.try_get(0)?,
        theme: Theme {
            primary_color_text: row.try_get(1)?,
            primary_color: row.try_get(2)?,
            secondary_color: row.try_get(3)?,
            background_color_primary: row.try_get(4)?,
            background_color_secondary: row.try_get(5)?,
            background_color_tertiary: row.try_get(6)?,
            primary_grey: row.try_get(7)?,
            secondary_grey: row.try_get(8)?,
            font_size: row.try_get(9)?,
            transparency: row.try_get(10)?,
            transparency_value: row.try_get(11)?,
            transparency_blur: row.try_get(12)?,
        },
    })
}