use crate::db::auth::auth::User;

use super::cache_manager::CacheManager;

lazy_static::lazy_static! {
    pub static ref USER_CACHE: CacheManager<i64, User> = CacheManager::new(500);
}