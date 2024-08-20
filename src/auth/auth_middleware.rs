use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::header::AUTHORIZATION,
    Error, HttpResponse,
};
use actix_web_lab::middleware::Next;
use serde_json::json;

use crate::{cache::init_caches::USER_CACHE, db::auth::auth::Database};

use super::utils::TokenHandler;

use crate::error_response;

pub async fn check_auth_mw<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<BoxBody>, Error>
where
    B: MessageBody + 'static,
{
    if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
        let auth_header = auth_header.to_str().unwrap();

        let token_handler = TokenHandler::new().await;

        match token_handler.verify_token(auth_header).await {
            Ok(claims) => {
                let uid: i64 = match claims.user_id.parse() {
                    Ok(uid) => uid,
                    Err(e) => {
                        let http_res = error_response!(400, e.to_string()).map_into_boxed_body();
                        let (req, _pl) = req.into_parts();
                        let service_res = ServiceResponse::new(req, http_res);
                        return Ok(service_res)
                    }
                };

                let cache = &*USER_CACHE;
                if let Some(user_details) = cache.get(&uid) { // check the cache before calling the db
                    if !user_details.email_verified {
                        let http_res = error_response!(403, "Verify your email before using the API service.").map_into_boxed_body();
                        let (req, _pl) = req.into_parts();
                        let service_res = ServiceResponse::new(req, http_res);
                        return Ok(service_res);
                    }
                } else {
                    let db = match Database::new().await {
                        Ok(db) => db,
                        Err(e) => {
                            let http_res = error_response!(500, e.to_string()).map_into_boxed_body();
                            let (req, _pl) = req.into_parts();
                            let service_res = ServiceResponse::new(req, http_res);
                            return Ok(service_res);
                        }
                    };

                    match db.create_table().await {
                        Ok(()) => (),
                        Err(e) => {
                            let http_res = error_response!(500, e.to_string()).map_into_boxed_body();
                            let (req, _pl) = req.into_parts();
                            let service_res = ServiceResponse::new(req, http_res);
                            return Ok(service_res);
                        }
                    }

                    let user_details = match db.read_by_uid(uid).await {
                        Ok(Some(details)) => details,
                        Ok(None) => {
                            let http_res = error_response!(404, "No user ID found associated with this token").map_into_boxed_body();
                            let (req, _pl) = req.into_parts();
                            let service_res = ServiceResponse::new(req, http_res);
                            return Ok(service_res);
                        }
                        Err(e) => {
                            let http_res = error_response!(500, e.to_string()).map_into_boxed_body();
                            let (req, _pl) = req.into_parts();
                            let service_res = ServiceResponse::new(req, http_res);
                            return Ok(service_res);
                        }
                    };

                    cache.insert(uid, user_details.clone());
                    if !user_details.email_verified {
                        let http_res = error_response!(403, "Verify your email before using the API service.").map_into_boxed_body();
                        let (req, _pl) = req.into_parts();
                        let service_res = ServiceResponse::new(req, http_res);
                        return Ok(service_res);
                    }
                }

            },
            Err(e) => {
                println!("{:?}", e);

                let message = format!("403: {}", e.to_string());

                let http_res = HttpResponse::Forbidden()
                    .json(json!({"error": message}))
                    .map_into_boxed_body();

                let (req, _pl) = req.into_parts();

                let service_res = ServiceResponse::new(req, http_res);

                return Ok(service_res)
            }
        }

    } else {
        let message = "403: Authorization header missing!";
        let http_res = HttpResponse::Forbidden()
            .json(json!({"error": message}))
            .map_into_boxed_body();

        let (req, _pl) = req.into_parts();

        let service_res = ServiceResponse::new(req, http_res);

        return Ok(service_res);
    }


    let res = next.call(req).await?;
    Ok(res.map_body(|_, body| BoxBody::new(body)))
}