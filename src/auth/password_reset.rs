use actix_web::{post, HttpResponse};

use bcrypt::{hash, DEFAULT_COST};
use lettre::{message::SinglePart, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use serde_json::json;

use crate::{auth::utils::{validate_password, CodeStorage, TokenHandler}, db::auth::auth::Database, error::ActixError, secrets::SECRETS};



#[post("/request_reset_password")]
pub async fn request_reset_password(req_body: String) -> Result<HttpResponse, ActixError> {
    #[derive(Debug, Deserialize)]
    struct Email {
        email: String
    }

    let Email { email } = serde_json::from_str(&req_body)
        .map_err(|e| ActixError::JsonError(e.to_string()))?;

    let auth_user_db = Database::new().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;
    
    auth_user_db.create_table().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    let auth_user = auth_user_db.read_by_email(&email).await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    if let Some(user) = auth_user {

        if !user.email_verified {
            return Ok(HttpResponse::Conflict().json(json!({"error": "you first need to verify your email to confirm you own this email address before requesting a password change."})));
        }

        let generate_code = CodeStorage::PasswordResetCodes;
        let code = generate_code.create(&user.uid.to_string())
            .map_err(|e| ActixError::CodeGenError(e.to_string()))?;

        println!("code: {}", code);

        match send_password_reset_code_email(&code, &user.email) {
            Ok(()) => {},
            Err(e) => return Ok(HttpResponse::InternalServerError().json(json!({"error": e.to_string()})))
        }

        Ok(HttpResponse::Ok().json(json!({"message": "password reset email sent."})))
    } else {
        Ok(HttpResponse::NotFound().json(json!({"error": format!("couldnt find an user associated with the email '{}'", email)})))
    }

}

const EMAIL_RESET_PASSWORD_BODY: &str = include_str!("password_reset_body.html");

fn send_password_reset_code_email(code: &str, email: &str) -> anyhow::Result<()> {
    let body = EMAIL_RESET_PASSWORD_BODY.replace("{code}", code);

    let email = Message::builder()
        .from(SECRETS.get("NO_REPLY_EMAIL").unwrap().parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Your Password Change request code for acid4sigmas")
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



#[post("/reset_password")]
pub async fn reset_password(req_body: String) -> Result<HttpResponse, ActixError> {

    #[derive(Debug, Deserialize)]
    struct ResetPassword {
        email: String,
        code: u64,
        new_password: String
    }

    let ResetPassword { email, code, new_password } = serde_json::from_str(&req_body)
        .map_err(|e| ActixError::JsonError(e.to_string()))?;

    let auth_user_db = Database::new().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;
    
    auth_user_db.create_table().await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    let auth_user = auth_user_db.read_by_email(&email).await
        .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

    if let Some(user) = auth_user {
        let code_storage = CodeStorage::PasswordResetCodes;

        let code_s = code_storage.get_code(&user.uid.to_string());

        if let Some(stored_code) = code_s {
            if stored_code == code.to_string() {

                match validate_password(&new_password) {
                    Ok(()) => (),
                    Err(e) => return Ok(HttpResponse::Conflict().json(json!({"error": e})))
                }

                let hashed_password = hash(&new_password, DEFAULT_COST).unwrap();

                auth_user_db.update_password(user.uid, &hashed_password).await
                    .map_err(|e| ActixError::DatabaseError(e.to_string()))?;

                code_storage.delete_code(&user.uid.to_string());

                TokenHandler::new().await.destroy_all_tokens(user.uid).await.unwrap();

                match send_password_changed_email(&user.email) {
                    Ok(()) => {},
                    Err(e) => return Ok(HttpResponse::InternalServerError().json(json!({"error": e.to_string()})))
                }

                return Ok(HttpResponse::Ok().json(json!({"message": "changed password successfully."})));
            } else {
                return Ok(HttpResponse::Unauthorized().json(json!({"erorr": "the authentication code is wrong"})));
            }
        } else {
            return Ok(HttpResponse::Conflict().json(json!({"error": "no pending verification code outgoing."})));
        }
    } else {
        return Ok(HttpResponse::NotFound().json(json!({"error": "no user associated with this email."})));
    }

}

const EMAIL_PASSWORD_CHANGED_BODY: &str = include_str!("password_changed_body.html");

fn send_password_changed_email(email: &str) -> anyhow::Result<()> {

    let body = EMAIL_PASSWORD_CHANGED_BODY.to_string();

    let email = Message::builder()
        .from(SECRETS.get("NO_REPLY_EMAIL").unwrap().parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Your Password Change request code for acid4sigmas")
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
