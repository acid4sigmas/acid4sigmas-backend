use std::{collections::HashMap, sync::Arc};

use actix_web::{get, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use get_repo::get_repo_info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{cache::init_caches::GITHUB_REPO_CACHE, error_response, secrets::SECRETS};

mod get_repo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repo {
    pub forks: u32,
    pub language: Option<String>,
    pub owner: Owner,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoInfo {
    pub languages: Option<HashMap<String, u64>>,
    pub repo: Repo
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Owner {
    pub login: String,
}

type SharedTimestamp = Arc<Mutex<DateTime<Utc>>>;

lazy_static::lazy_static! {
    static ref CACHE_REFRESH_TIMESTAMP: SharedTimestamp = Arc::new(Mutex::new(Utc::now()));
}

#[get("/repo")]
pub async fn get_repo_() -> HttpResponse {
    let now = Utc::now();

    let cache_timestamp = CACHE_REFRESH_TIMESTAMP.clone();

    let mut cached_timestamp = cache_timestamp.lock().await;

    let cache = &*GITHUB_REPO_CACHE;

    if now > *cached_timestamp {
        *cached_timestamp = now + Duration::minutes(10);

        drop(cached_timestamp);

        let repo = match get_repo_info(SECRETS.get("OWNER").unwrap(), SECRETS.get("REPO").unwrap()).await {
            Ok(repo) => repo,
            Err(e) => return error_response!(500, e.to_string())
        };

        cache.insert(0, repo.clone());

        println!("got repo from expired timestamp lol: {:?}", repo);

        return HttpResponse::Ok().json(repo)


    } else {
        drop(cached_timestamp);
        if let Some(cache) = cache.get(&0) {
            println!("got repo from cache: {:?}", cache);
            return HttpResponse::Ok().json(cache)
        } else {
            let repo = match get_repo_info(SECRETS.get("OWNER").unwrap(), SECRETS.get("REPO").unwrap()).await {
                Ok(repo) => repo,
                Err(e) => return error_response!(500, e.to_string())
            };
    
            cache.insert(0, repo.clone());

            println!("got repo from nothing in cache: {:?}", repo);

            return HttpResponse::Ok().json(repo)
        }
    }

    
}