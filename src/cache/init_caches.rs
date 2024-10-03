use crate::{
    db::{
        api::{
            cloudthemes::{cloudthemes::CloudTheme, status::CloudThemesStatus},
            me::User,
        },
        auth::auth::AuthUser,
    },
    pub_api::github::RepoInfo,
};

use super::cache_manager::CacheManager;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref USER_CACHE: CacheManager<i64, AuthUser> = CacheManager::new(500);
}

lazy_static! {
    pub static ref USER_ME_CACHE: CacheManager<i64, User> = CacheManager::new(500);
}

lazy_static! {
    pub static ref USER_CLOUDTHEMES: CacheManager<i64, CloudTheme> = CacheManager::new(600);
}

lazy_static! {
    pub static ref USER_CLOUDTHEMES_STATUS: CacheManager<i64, CloudThemesStatus> =
        CacheManager::new(600);
}

lazy_static! {
    pub static ref GITHUB_REPO_CACHE: CacheManager<i64, Vec<RepoInfo>> = CacheManager::new(10);
}
