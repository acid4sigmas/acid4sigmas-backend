use actix_web::{post, HttpResponse, Responder, web, ResponseError};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use std::fmt::{self, format};

#[derive(Debug, Error)]
pub enum ActixError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(String),

    #[error("Code gen error: {0}")]
    CodeGenError(String),

    #[error("Token gen error: {0}")]
    TokenGenError(String)
}  

impl ResponseError for ActixError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {

        let error_message = match self {
            ActixError::DatabaseError(err) => format!("Internal Server Error: {}", err),
            ActixError::JsonError(err) => format!("Bad Request: {}", err),
            ActixError::CodeGenError(err) => format!("Conflict: {}", err),
            ActixError::TokenGenError(err) => format!("Internal Server Error: {}", err)
        };

        HttpResponse::build(self.status_code()).json(json!({
            "error": error_message
        }))
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ActixError::DatabaseError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ActixError::JsonError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            ActixError::CodeGenError(_) => actix_web::http::StatusCode::CONFLICT,
            ActixError::TokenGenError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
