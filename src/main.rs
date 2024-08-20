use actix_web::{get,  web, App, HttpResponse, HttpServer, Responder};



mod auth;
mod db;
mod secrets;
mod error;
mod cache;

use auth::{auth_middleware::check_auth_mw, login, password_reset::{request_reset_password, reset_password}, register, send_verifiaction_email, verify_email};


use actix_files as fs; 

use actix_web_lab::middleware::from_fn;


#[macro_export]
macro_rules! error_response {
    ($status_code:expr, $message:expr) => {
        HttpResponse::build(actix_web::http::StatusCode::from_u16($status_code).unwrap())
            .json(json!({ "error": $message }))
    };
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(include_str!("../static/index.html"))
}

async fn nested_hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from the nested route!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/static", "static").show_files_listing())
            .service(
                web::scope("/api")
                    .wrap(from_fn(check_auth_mw))
                    .route("/nested", web::get().to(nested_hello))
            )   
            .service(
                web::scope("/auth")
                    .service(register)
                    .service(login)
                    .service(send_verifiaction_email)
                    .service(verify_email)
                    .service(request_reset_password)
                    .service(reset_password)
            )
            .service(index)
    })
    .bind("127.0.0.1:8080")? 
    .run()
    .await
}
