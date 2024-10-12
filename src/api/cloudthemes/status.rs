use crate::auth::utils::Claims;
use crate::cache::init_caches::USER_CLOUDTHEMES_STATUS;
use crate::db::api::cloudthemes::status::Database;
use crate::error_response;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse};

use crate::models::api::cloudtheme::CloudThemesStatus;

#[get("/cloudthemes/status")]
pub async fn get_cloudthemes_status(req: HttpRequest) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned().unwrap();

    let user_id = match claims.user_id.parse::<i64>() {
        Ok(uid) => uid,
        Err(e) => return error_response!(500, e.to_string()),
    };

    let cache = &*USER_CLOUDTHEMES_STATUS;
    if let Some(cloudthemes) = cache.get(&user_id) {
        return HttpResponse::Ok().json(cloudthemes);
    } else {
        let db = match Database::new().await {
            Ok(db) => db,
            Err(e) => return error_response!(500, e.to_string()),
        };

        match db.create_table().await {
            Ok(()) => (),
            Err(e) => return error_response!(500, e.to_string()),
        }

        match db.read_status_by_uid(user_id).await {
            Ok(status) => {
                cache.insert(user_id, status.clone());
                return HttpResponse::Ok().json(status);
            }
            Err(e) => return error_response!(500, e.to_string()),
        }
    }
}

#[post("/cloudthemes/status")]
pub async fn post_cloudthemes_status(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned().unwrap();

    let user_id = match claims.user_id.parse::<i64>() {
        Ok(uid) => uid,
        Err(e) => return error_response!(500, e.to_string()),
    };

    let body_str = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(e) => return error_response!(400, format!("Invalid UTF-8 sequence: {}", e)),
    };

    let status: CloudThemesStatus = match serde_json::from_str(&body_str) {
        Ok(status) => status,
        Err(e) => return error_response!(500, e.to_string()),
    };

    let db = match Database::new().await {
        Ok(db) => db,
        Err(e) => return error_response!(500, e.to_string()),
    };

    match db.create_table().await {
        Ok(()) => (),
        Err(e) => return error_response!(500, e.to_string()),
    }

    match db.update_status(user_id, status.enabled).await {
        Ok(()) => {
            let cache = &*USER_CLOUDTHEMES_STATUS;
            cache.remove(&user_id);
        }
        Err(e) => return error_response!(500, e.to_string()),
    }

    HttpResponse::Ok().json(status)
}
