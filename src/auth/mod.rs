use actix_web::{post, HttpResponse};

use bcrypt::{hash, verify, DEFAULT_COST};
use lettre::{message::SinglePart, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use utils::{generate_uid, validate_password, validate_username, CodeStorage, TokenHandler, UsernameOrEmail};

pub mod password_reset;
pub mod auth_middleware;
pub mod utils;

use crate::cache::init_caches::{USER_CACHE, USER_ME_CACHE};
use crate::{db::auth::auth::Database, secrets::SECRETS};

use crate::db::api::me::Database as UserDatabase;


use crate::{error_response, message_response, token_response};

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

    match validate_username(&json_content.username) {
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
            
        token_response!(token)
    } else {
        error_response!(500, "failed to verify hash")
    }

}

#[post("/login")]
pub async fn login(req_body: String) -> HttpResponse {

    #[derive(Debug, Deserialize)]
    struct LoginRequest {
        username_or_email: String,
        password: String
    } 

    let json_content: LoginRequest = match serde_json::from_str(&req_body) {
        Ok(result) => result,
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



    let auth_user = match UsernameOrEmail::parse(&json_content.username_or_email) {
        UsernameOrEmail::Email(email) => {
            match auth_user_db.read_by_email(&email).await {
                Ok(user) => user,
                Err(e) => return error_response!(500, e.to_string())
            }
        }
        UsernameOrEmail::Username(username) => {
            match auth_user_db.read_by_username(&username).await {
                Ok(user) => user,
                Err(e) => return error_response!(500, e.to_string())
            }
        }
    };

    if let Some(user) = auth_user {
        let verify = verify(json_content.password, &user.password_hash).unwrap();

        if verify {
            let token = match TokenHandler::new().await.generate_token(user.uid).await {
                Ok(token) => token,
                Err(e) => return error_response!(403, e.to_string())
            };
            return token_response!(token)
        } else {
            return error_response!(403, "password or username is wrong");
        }

    } else {
        return error_response!(404, "couldnt find a user associated with this username or email");
    }
}

#[derive(Debug, Deserialize)]
pub struct Token {
    pub token: String
}

#[post("/send_verification_email")]
pub async fn send_verifiaction_email(req_body: String) -> HttpResponse {

    let Token { token } = match serde_json::from_str(&req_body) {
        Ok(token) => token,
        Err(e) => return error_response!(400, e.to_string())
    };

    let claims = match TokenHandler::new().await.verify_token(&token).await {
        Ok(result) => result,
        Err(e) => return error_response!(403, e.to_string())
    };

    let user_id = claims.user_id.clone();

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

    let auth_user = match auth_user_db.read_by_uid(user_id.parse().unwrap()).await {
        Ok(user) => user,
        Err(e) => return error_response!(500, e.to_string())
    };

    match auth_user {
        Some(user) => {
            if user.email_verified {
                let cache = &*USER_CACHE;

                let result = cache.remove(&user.uid);

                return error_response!(409, "your email is already verified");
            } else {

                let code_gen = CodeStorage::EmailVerificationCodes;

                let code = match code_gen.create(&user.uid.to_string()) {
                    Ok(code) =>  code,
                    Err(e) => return error_response!(502, e.to_string())
                };


                match send_email(&code, &user.email) {
                    Ok(()) => {},
                    Err(e) => return error_response!(502, e.to_string())
                }
                return message_response!("Verification email sent.");
                
            }
        },
        None => return error_response!(404, "Couldn't find a user associated with this token.")
    }

}

const EMAIL_VERIFY_BODY: &str = include_str!("verify_email_body.html");

fn send_email(code: &str, email: &str) -> anyhow::Result<()> {
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
pub async fn verify_email(req_body: String) -> HttpResponse {
    #[derive(Debug, Deserialize)]
    struct VerifyEmail {
        token: String,
        code: u64
    }

    let VerifyEmail { token, code } = match serde_json::from_str(&req_body) {
        Ok(result) => result,
        Err(e) => return error_response!(400, e.to_string())
    };

    let claims = match TokenHandler::new().await.verify_token(&token).await {
        Ok(result) => result,
        Err(e) => return error_response!(403, e.to_string())
    };

    let user_id = claims.user_id.clone();

   
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

    let auth_user = match auth_user_db.read_by_uid(user_id.parse().unwrap()).await {
        Ok(user) => user,
        Err(e) => return error_response!(500, e.to_string())
    };


    if let Some(user) = auth_user {
        let code_storage = CodeStorage::EmailVerificationCodes;

        let code_ = code_storage.get_code(&user_id);

        if let Some(stored_code) = code_ {
            if stored_code == code.to_string() {
                code_storage.delete_code(&user_id);

                match auth_user_db.update_email_verification(user.uid, true).await {
                    Ok(()) => (),
                    Err(e) => return error_response!(500, e.to_string())
                }

                                    
                let cache = &*USER_CACHE;

                let _ = cache.remove(&user.uid);

                let cache_api = &*USER_ME_CACHE;

                let _ = cache_api.remove(&user.uid);

                let generated_token = match TokenHandler::new().await.generate_token(user.uid).await {
                    Ok(token) => token,
                    Err(e) => return error_response!(403, e.to_string())
                };

                return token_response!(generated_token);
            } else {
                return error_response!(403, "the authentication code is wrong");
            }
        } else {
            return error_response!(409, "no pending verification code outgoing");
        }
    } else {
        
        return error_response!(404, "no user associated with this token.")
    }
 
}