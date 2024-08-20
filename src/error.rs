use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;


#[derive(Debug, Error)]
pub enum ActixError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(String),

    #[error("Code gen error: {0}")]
    CodeGenError(String),

}  

impl ResponseError for ActixError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {

        let error_message = match self {
            ActixError::DatabaseError(err) => format!("Internal Server Error: {}", err),
            ActixError::JsonError(err) => format!("Bad Request: {}", err),
            ActixError::CodeGenError(err) => format!("Conflict: {}", err),
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
        }
    }
}
