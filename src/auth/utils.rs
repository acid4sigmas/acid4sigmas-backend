use std::{collections::HashMap, sync::Mutex, time::{Duration, SystemTime, UNIX_EPOCH}};
use anyhow::anyhow;
use chrono::{Utc, Duration as ChronoDuration};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::secrets::SECRETS;

use crate::db::auth::tokens::Database as TokenCheckDatabase;


pub fn generate_uid() -> i64 {
    let epoch = 1_704_037_200_000;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let timestamp = now - epoch;

    let timestamp_part = (timestamp & 0x3FFFFFFFFFF) << 22;

    let machine_id = rand::thread_rng().gen_range(0..1024);
    let machine_id_part = (machine_id & 0x3FF) << 12;

    let sequence = rand::thread_rng().gen_range(0..4096);

    let uid = timestamp_part | machine_id_part | sequence;
    uid as i64
}

pub static PASSWORD_RESET_CODE_STORE: Lazy<Mutex<HashMap<String, (String, f64)>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub static EMAIL_VERIFICATION_CODE_STORE: Lazy<Mutex<HashMap<String, (String, f64)>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub enum CodeStorage {
    EmailVerificationCodes,
    PasswordResetCodes,
}

impl CodeStorage {
    fn get_store(&self) -> &Lazy<Mutex<HashMap<String, (String, f64)>>> {
        match self {
            CodeStorage::EmailVerificationCodes => &EMAIL_VERIFICATION_CODE_STORE,
            CodeStorage::PasswordResetCodes => &PASSWORD_RESET_CODE_STORE
        }
    }

    fn insert_code(&self, user_id: &str, code: String) {
        let expires_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::new(600, 0);
        self.get_store().lock().unwrap().insert(user_id.to_string(), (code, expires_at.as_secs() as f64));
    }

    fn generate_verification_code(&self) -> String {
        let code: u32 = rand::thread_rng().gen_range(100000..999999);
        code.to_string()
    }

    pub fn has_pending_code(&self, user_id: &str) -> bool {
        let store = self.get_store().lock().unwrap();
        store.contains_key(user_id)
    }

    pub fn get_retry_time(&self, user_id: &str) -> Option<f64> {
        let store = self.get_store().lock().unwrap();
        if let Some((_, expires_at)) = store.get(user_id) {
            let current_time: f64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
            let remaining_time = expires_at - current_time;
            if remaining_time < 540.0 {
                None
            } else {
                Some(remaining_time - 540.0)
            }
        } else {
            None
        }
    }

    pub fn create(&self, user_id: &str) -> Result<String, String> {
        if self.has_pending_code(user_id) {
            if let Some(retry_time) = self.get_retry_time(user_id) {
                return Err(format!("Please wait {:?} seconds before requesting a new code", retry_time));               
            }
        }

        let code = self.generate_verification_code();
        self.insert_code(user_id, code.clone());
        Ok(code)
    }

    pub fn get_code(&self, user_id: &str) -> Option<String> {
        let mut store = self.get_store().lock().unwrap();
        if let Some((code, expires_at)) = store.get(user_id) {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
            if current_time <= *expires_at {
                Some(code.clone())
            } else {
                store.remove(user_id);
                None
            }
        } else {
            None
        }
    }

    pub fn delete_code(&self, user_id: &str) {
        self.get_store().lock().unwrap().remove(user_id);
    }

}

pub fn validate_password(password: &str) -> Result<(), String> {
    let min_length = 8;
    let digit_regex = Regex::new(r"\d").unwrap();
    let uppercase_regex = Regex::new(r"[A-Z]").unwrap(); 
    let lowercase_regex = Regex::new(r"[a-z]").unwrap(); 
    let special_char_regex = Regex::new(r"[!@#$%^&*()\-=+?]").unwrap(); 

    if password.len() < min_length {
        return Err(format!("password must be at least {} chars long", min_length));
    }

    if !digit_regex.is_match(password) {
        return Err(String::from("password must contain at least one digit."));
    }

    if !uppercase_regex.is_match(password) {
        return Err(String::from("password must contain at least one uppercase letter."));
    }

    if !lowercase_regex.is_match(password) {
        return Err(String::from("password must contain at least one lowercase letter."));
    }

    if !special_char_regex.is_match(password) {
        return Err(String::from("password must contain at least one special character. allowed special characters: !@#$%^&*()-_=+?"));
    }
    
    Ok(())
}

#[derive(Debug)]
pub enum UsernameOrEmail {
    Email(String),
    Username(String),
}

impl UsernameOrEmail {
    pub fn parse(input: &str) -> Self {
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();

        if email_regex.is_match(input) {
            UsernameOrEmail::Email(input.to_string())
        } else {
            UsernameOrEmail::Username(input.to_string())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub exp: usize, 
    pub jti: String
}

pub struct TokenHandler {
    secret_key: Vec<u8>,
    db: TokenCheckDatabase
}

impl TokenHandler {
    
    fn get_secret_key() -> Vec<u8> {
        SECRETS.get("SECRET_KEY")
            .expect("SECRET_KEY not found")
            .as_bytes()
            .to_vec()
    }

    pub async fn new() -> Self {
        let db = TokenCheckDatabase::new().await.unwrap();
        db.create_table().await.unwrap();
        TokenHandler {
            secret_key: Self::get_secret_key(),
            db
        }
    }

    pub async fn generate_token(&self, user_id: i64) -> anyhow::Result<String> {
        let expiration = Utc::now() + ChronoDuration::days(365);

        let claims = Claims {
            user_id: user_id.to_string(),
            exp: expiration.timestamp() as usize,
            jti: Uuid::new_v4().to_string()
        };
        let db = &self.db;

        db.create_table().await?;
        db.insert(user_id, &claims.jti, expiration).await?;

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.secret_key),
        )
        .expect("Failed to generate token");

        Ok(token)

    }

    pub async fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);

        let db = &self.db;

        match decode::<Claims>(token, &DecodingKey::from_secret(&self.secret_key), &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;

                let jtis = db.read_by_uid(claims.user_id.parse().map_err(|_| anyhow!("Invalid user ID in claims"))?).await.unwrap();

                let mut is_jti_valid = false;

                for jti in jtis {
                    if jti == claims.jti {
                        is_jti_valid = true;
                        break;
                    }
                }

                if claims.exp < Utc::now().timestamp() as usize {
                    return Err(anyhow!("your token is expired")); 
                }

                if is_jti_valid {
                    Ok(claims)
                } else {
                    Err(anyhow!("No valid jti found for this token"))
                }
            },
            Err(_e) => Err(anyhow!("Failed to validate token")),
        }
    }

    pub async fn destroy_all_tokens(&self, user_id: i64) -> anyhow::Result<()> {
        let db = &self.db;
        db.create_table().await?;
        db.delete_by_uid(user_id).await?;

        Ok(())
    }
}
