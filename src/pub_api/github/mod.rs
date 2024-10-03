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
    pub name: String,
    pub forks: u32,
    pub language: Option<String>,
    pub owner: Owner,
    pub html_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoInfo {
    pub languages: Option<HashMap<String, u64>>,
    pub repo: Repo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Owner {
    pub login: String,
    pub html_url: String,
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

    let repos: Vec<String> = SECRETS
        .get("REPO")
        .unwrap()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    if now > *cached_timestamp {
        *cached_timestamp = now + Duration::minutes(10);

        drop(cached_timestamp);

        let mut repos_vec: Vec<RepoInfo> = Vec::new();

        for repo in repos {
            let repo = match get_repo_info(SECRETS.get("OWNER").unwrap(), &repo).await {
                Ok(repo) => repo,
                Err(e) => return error_response!(500, e.to_string()),
            };

            repos_vec.push(repo);
        }

        cache.insert(0, repos_vec.clone());

        return HttpResponse::Ok().json(repos_vec);
    } else {
        drop(cached_timestamp);
        if let Some(cache) = cache.get(&0) {
            return HttpResponse::Ok().json(cache);
        } else {
            let mut repos_vec: Vec<RepoInfo> = Vec::new();

            for repo in repos {
                let repo = match get_repo_info(SECRETS.get("OWNER").unwrap(), &repo).await {
                    Ok(repo) => repo,
                    Err(e) => return error_response!(500, e.to_string()),
                };

                repos_vec.push(repo);
            }

            cache.insert(0, repos_vec.clone());

            return HttpResponse::Ok().json(repos_vec);
        }
    }
}
