#![allow(deprecated)]

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod api;
mod auth;
mod cache;
mod db;
mod error;
mod pub_api;
mod secrets;
mod util;

use api::{
    cloudthemes::{
        cloudthemes::{get_cloudthemes, set_cloudtheme},
        status::{get_cloudthemes_status, post_cloudthemes_status},
    },
    me::me,
};
use auth::{
    auth_middleware::check_auth_mw,
    login,
    password_reset::{request_reset_password, reset_password},
    register, send_verifiaction_email, verify_email,
};

use actix_files as fs;

use actix_cors::Cors;
use actix_web_lab::middleware::from_fn;
use pub_api::faith::book::faith_book;
use pub_api::github::get_repo_;

#[macro_export]
macro_rules! error_response {
    ($status_code:expr, $message:expr) => {
        HttpResponse::build(actix_web::http::StatusCode::from_u16($status_code).unwrap())
            .json(serde_json::json!({ "error": $message }))
    };
}

#[macro_export]
macro_rules! message_response {
    ($message:expr) => {
        HttpResponse::build(actix_web::http::StatusCode::OK)
            .json(serde_json::json!({"message": $message}))
    }
}

#[macro_export]
macro_rules! token_response {
    ($token:expr) => {
        HttpResponse::build(actix_web::http::StatusCode::OK)
            .json(serde_json::json!({"token": $token}))
    }
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../static/index.html"))
}

async fn nested_hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from the nested route!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_origin()
            .allow_any_method();

        App::new()
            .wrap(cors)
            .service(fs::Files::new("/static", "static").show_files_listing())
            .service(fs::Files::new("/assets", "assets").show_files_listing())
            .service(
                web::scope("/api")
                    .wrap(from_fn(check_auth_mw))
                    .route("/nested", web::get().to(nested_hello))
                    .service(me)
                    .service(set_cloudtheme)
                    .service(get_cloudthemes)
                    .service(get_cloudthemes_status)
                    .service(post_cloudthemes_status),
            )
            .service(
                web::scope("/pub_api")
                    .service(get_repo_)
                    .service(faith_book),
            )
            .service(
                web::scope("/auth")
                    .service(register)
                    .service(login)
                    .service(send_verifiaction_email)
                    .service(verify_email)
                    .service(request_reset_password)
                    .service(reset_password),
            )
            .service(index)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
