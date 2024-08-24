use actix_web::{get, HttpMessage, HttpRequest, HttpResponse};
use serde_json::json;
use crate::{auth::utils::Claims, cache::init_caches::USER_ME_CACHE, db::api::me::Database, error_response};

#[get("/me")]
pub async fn me(req: HttpRequest) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned().unwrap();

    let user_id = match claims.user_id.parse::<i64>() {
        Ok(uid) => uid,
        Err(e) => return error_response!(500, e.to_string())
    };

    let cache = &*USER_ME_CACHE;
    if let Some(user) = cache.get(&user_id) {
        return HttpResponse::Ok().json(user);
    } else {
        let db = match Database::new().await {
            Ok(db) => db,
            Err(e) => return error_response!(500, e.to_string())
        };
    
        match db.create_table().await {
            Ok(()) => (),
            Err(e) => return error_response!(500, e.to_string())
        }
    
        let user_details = match db.read_by_uid(claims.user_id.parse().unwrap()).await {
            Ok(user) => user,
            Err(e) => return error_response!(500, e.to_string())
        };
    
        if let Some(usr_details) = user_details {
            return HttpResponse::Ok().json(usr_details);
        } else {
            return error_response!(404, "couldnt find a user with this uid");
        }
    }
    
}