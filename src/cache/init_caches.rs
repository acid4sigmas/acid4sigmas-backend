use crate::db::{api::{cloudthemes::CloudTheme, me::User}, auth::auth::AuthUser};

use super::cache_manager::CacheManager;

lazy_static::lazy_static! {
    pub static ref USER_CACHE: CacheManager<i64, AuthUser> = CacheManager::new(500);
}

lazy_static::lazy_static! {
    pub static ref USER_ME_CACHE: CacheManager<i64, User> = CacheManager::new(500);
}

lazy_static::lazy_static! {
    pub static ref USER_CLOUDTHEMES: CacheManager<i64, CloudTheme> = CacheManager::new(600);
}