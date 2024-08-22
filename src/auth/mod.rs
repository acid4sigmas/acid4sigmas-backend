use actix_web::{post, HttpResponse, Result};

use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lettre::{message::SinglePart, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use serde_json::json;
use utils::{generate_uid, validate_password, Claims, CodeStorage, TokenHandler, UsernameOrEmail};
use uuid::Uuid;

pub mod password_reset;
pub mod auth_middleware;
pub mod utils;

use crate::cache::init_caches::USER_CACHE;
use crate::{db::auth::auth::Database, error::ActixError, secrets::SECRETS};

use crate::db::api::me::Database as UserDatabase;


use crate::error_response;

fn get_secret_key() -> Vec<u8> {
    SECRETS.get("SECRET_KEY").unwrap().as_bytes().to_vec()
}


async fn generate_token(user_id: i64) -> String {

    use crate::db::auth::tokens::Database as TokenCheckDatabase;

    let expiration = chrono::Utc::now() + chrono::Duration::days(365);

    let claims = Claims {
        user_id: user_id.to_string(),
        exp: expiration.timestamp() as usize,
        jti: Uuid::new_v4().to_string()
    };

    let db = TokenCheckDatabase::new().await.unwrap();

    db.create_table().await.unwrap();

    db.insert(user_id, &claims.jti, expiration).await.unwrap();


    let algorithm = Algorithm::HS256;

    let secret_key = get_secret_key();

    let token = encode(
        &Header::new(algorithm),
        &claims,
        &EncodingKey::from_secret(&secret_key),
    )
    .expect("Failed to generate token");

    token
}

fn verify_token(token: &str) -> Result<(bool, Claims), &'static str> {
    let secret_key = get_secret_key();

    let algorithm = Algorithm::HS256;

    let validation = Validation::new(algorithm);

    let decoding_key = DecodingKey::from_secret(&secret_key);

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            Ok((true, token_data.claims))
        }
        Err(_e) => {
            Err("failed to validate token")
        }
    }
}


#[post("/register")]
pub async fn register(req_body: String) -> HttpResponse {

    #[derive(Debug, Deserialize)]
    struct RegisterRequest {
        username: String,
        password: String,
        email: String,
    }

    let json_content: RegisterRequest = match serde_json::from_str(&req_body) {
        Ok(json) => json,
        Err(e) => return error_response!(400, e.to_string())
    };


    let auth_user_db = match Database::new().await {
        Ok(db) => db,
        Err(e) => {
            return error_response!(500, e.to_string())
        }
    };
       
    match auth_user_db.create_table().await {
        Ok(()) => (),
        Err(e) => return error_response!(500, e.to_string())
    }
    
        
    if let Some(user) = match auth_user_db.read_by_email(&json_content.email).await {
        Ok(user) => user,
        Err(e) => return error_response!(500, e.to_string())
    } {
        return error_response!(409, format!("email '{}' is already registered, try to login instead!", user.email))
    };

    if let Some(user) = match auth_user_db.read_by_username(&json_content.username).await {
        Ok(user) => user,
        Err(e) => return error_response!(500, e.to_string())
    } {
        return error_response!(409, format!("username '{}' is already taken.", user.username))
    }


    match validate_password(&json_content.password) {
        Ok(()) => (),
        Err(e) => return error_response!(403, e.to_string())
    }

    let hashed = hash(&json_content.password, DEFAULT_COST).unwrap();
    let verified = verify(&json_content.password, &hashed).unwrap();

    if verified {
        let uid = generate_uid();

        match auth_user_db.insert(uid, &json_content.username, &hashed, &json_content.email).await {
            Ok(()) => (),
            Err(e) => return error_response!(500, e.to_string())
        }

        let user_db = match UserDatabase::new().await {
            Ok(db) => db,
            Err(e) => return error_response!(500, e.to_string())
        };

        match user_db.create_table().await {
            Ok(()) => (),
            Err(e) => return error_response!(500, e.to_string())
        }

        match user_db.insert(uid, &json_content.username, &json_content.email).await {
            Ok(()) => (),
            Err(e) => return error_response!(500, e.to_string())
        }


        let token = match TokenHandler::new().await.generate_token(uid).await {
            Ok(token) => token,
            Err(e) => return error_response!(403, e.to_string())
        }; 
            
        HttpResponse::Ok().json(serde_json::json!({"token": token}))
    } else {
        error_response!(500, "failed to verify hash")
    }

}

#[post("/login")]
pub async fn login(req_body: String) -> Result<HttpResponse, ActixError> {

    #[derive(Debug, Deserialize)]
    struct LoginRequest {
        username_or_email: String,
        password: String
    } 

    let json_content: LoginRequest = serde_json::from_str(&req_body)
        .map_err(|e| ActixError::JsonError(e.to_string()))?;

    

    let auth_user_db = Database::new().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    auth_user_db.create_table().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    let auth_user = match UsernameOrEmail::parse(&json_content.username_or_email) {
        UsernameOrEmail::Email(email) => {
            auth_user_db.read_by_email(&email).await
                .map_err(|e| ActixError::DatabaseError(e.to_string()))?
        }
        UsernameOrEmail::Username(username) => {
            auth_user_db.read_by_username(&username).await
                .map_err(|e| ActixError::DatabaseError(e.to_string()))?
        }
    };

    if let Some(user) = auth_user {
        let verify = verify(json_content.password, &user.password_hash).unwrap();

        if verify {
            let token = generate_token(user.uid).await;
            return Ok(HttpResponse::Ok().json(serde_json::json!({"token": token})));
        } else {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({"error": "password or username is wrong"})));
        }

    } else {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({"error": "password or username is wrong"})));
    }
}

#[derive(Debug, Deserialize)]
pub struct Token {
    pub token: String
}

#[post("/send_verification_email")]
pub async fn send_verifiaction_email(req_body: String) -> Result<HttpResponse, ActixError> {

    let Token { token } = serde_json::from_str(&req_body)
        .map_err(|e| ActixError::JsonError(e.to_string()))?;

    let (is_valid, claims) = match verify_token(&token) {
        Ok(result) => result,
        Err(e) => return Ok(HttpResponse::Unauthorized().json(json!({"error": e}))),
    };

    if !is_valid {
        return Ok(HttpResponse::Unauthorized().json(json!({"error": "Failed to authenticate"})));
    }

    let user_id = claims.user_id.clone();

    let auth_user_db = Database::new().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;
        
    auth_user_db.create_table().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    let auth_user = auth_user_db.read_by_uid(user_id.parse().unwrap()).await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    match auth_user {
        Some(user) => {
            if user.email_verified {
                return Ok(HttpResponse::Conflict().json(json!({"error": "Your email is already verified!"})));
            } else {

                let code_gen = CodeStorage::EmailVerificationCodes;

                let code = code_gen.create(&user.uid.to_string())
                    .map_err(|e| ActixError::CodeGenError(e.to_string()))?;

                match send_email(&code, &user.email) {
                    Ok(()) => {},
                    Err(e) => return Ok(HttpResponse::InternalServerError().json(json!({"error": e.to_string()})))
                }
                return Ok(HttpResponse::Ok().body("Verification email sent."))
                
            }
        },
        None => Ok(HttpResponse::NotFound().json(json!({"error": "Couldn't find a user associated with this token."}))),
    }

}

const EMAIL_VERIFY_BODY: &str = include_str!("verify_email_body.html");

fn send_email(code: &str, email: &str) -> anyhow::Result<()> {

    println!("{}", email);

    let body = EMAIL_VERIFY_BODY.replace("{code}", code);

    let email = Message::builder()
        .from(SECRETS.get("NO_REPLY_EMAIL").unwrap().parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Your Account Verificatio Code for acid4sigmas")
        .singlepart(SinglePart::html(body))?;

    let username = SECRETS.get("SMTP_USERNAME").unwrap();
    let password = SECRETS.get("SMTP_PASSWORD").unwrap();
    let smtp_relay = SECRETS.get("SMTP_RELAY").unwrap();

    let creds = Credentials::new(username.clone(), password.clone());

    let mailer = SmtpTransport::relay(&smtp_relay)?
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    
    Ok(())
}

#[post("/verify_email")]
pub async fn verify_email(req_body: String) -> Result<HttpResponse, ActixError> {
    #[derive(Debug, Deserialize)]
    struct VerifyEmail {
        token: String,
        code: u64
    }

    let json_content: VerifyEmail = serde_json::from_str(&req_body).map_err(|e| ActixError::JsonError(e.to_string()))?;

    let (is_valid, claims) = match verify_token(&json_content.token) {
        Ok(claims) => claims,
        Err(e) => return Ok(HttpResponse::Unauthorized().json(json!({"error": e})))
    };

    if !is_valid {
        return Ok(HttpResponse::Unauthorized().json(json!({"error": "Failed to authenticate"})));
    }

    let user_id = claims.user_id.clone();

    let auth_user_db = Database::new().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;
        
    auth_user_db.create_table().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    let auth_user = auth_user_db.read_by_uid(user_id.parse().unwrap()).await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    if let Some(user) = auth_user {
        let code_storage = CodeStorage::EmailVerificationCodes;

        let code = code_storage.get_code(&user_id);

        if let Some(stored_code) = code {
            if stored_code == json_content.code.to_string() {
                code_storage.delete_code(&user_id);

                auth_user_db.update_email_verification(user.uid, true).await
                    .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

                
                let cache = &*USER_CACHE;

                let _ = cache.remove(&user.uid);

                let generate_token = generate_token(user.uid).await;

                return Ok(HttpResponse::Ok().json(json!({"token": generate_token})))
            } else {
                return Ok(HttpResponse::Unauthorized().json(json!({"erorr": "the authentication code is wrong"})));
            }
        } else {
            return Ok(HttpResponse::Conflict().json(json!({"error": "no pending verification code outgoing."})));
        }
    } else {
        return Ok(HttpResponse::NotFound().json(json!({"error": "no user associated with this token."})));
    }

}