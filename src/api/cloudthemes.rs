use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse};

use crate::{auth::utils::Claims, cache::init_caches::USER_CLOUDTHEMES, db::api::cloudthemes::{Database, Theme}, error_response};

#[post("/cloudthemes")]
pub async fn set_cloudtheme(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned().unwrap();
    
    let user_id = match claims.user_id.parse::<i64>() {
        Ok(uid) => uid,
        Err(e) => return error_response!(500, e.to_string())
    };



    let body_str = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(e) => return error_response!(400, format!("Invalid UTF-8 sequence: {}", e)),
    };

    let theme: Theme = match serde_json::from_str(&body_str) {
        Ok(theme) => theme,
        Err(e) => return error_response!(400, format!("Failed to parse JSON: {}", e)),
    };

    println!("theme: {:?}", theme);

    let db = match Database::new().await {
        Ok(db) => db,
        Err(e) => return error_response!(500, e.to_string())
    };

    match db.create_table().await {
        Ok(()) => (),
        Err(e) => return error_response!(500, e.to_string())
    }

    match db.insert(user_id, theme).await {
        Ok(()) => (),
        Err(e) => return error_response!(500, e.to_string())
    };

    HttpResponse::Ok().finish()
}

#[get("/cloudthemes")]
pub async fn get_cloudthemes(req: HttpRequest) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned().unwrap();
    
    let user_id = match claims.user_id.parse::<i64>() {
        Ok(uid) => uid,
        Err(e) => return error_response!(500, e.to_string())
    };

    let cache = &*USER_CLOUDTHEMES;
    if let Some(cloudtheme) = cache.get(&user_id) {
        return HttpResponse::Ok().json(cloudtheme)
    } else {
        let db = match Database::new().await {
            Ok(db) => db,
            Err(e) => return error_response!(500, e.to_string())
        };

        match db.create_table().await {
            Ok(()) => (),
            Err(e) => return error_response!(500, e.to_string())
        }

        let theme = match db.read_by_uid(user_id).await {
            Ok(theme) => theme,
            Err(e) => return error_response!(500, e.to_string())
        };

        if let Some(theme) = theme {
            cache.insert(user_id , theme.clone());
            
            return HttpResponse::Ok().json(theme)
        } else {
            return error_response!(404, "no theme found for this uid ");
        }
        
    }

}