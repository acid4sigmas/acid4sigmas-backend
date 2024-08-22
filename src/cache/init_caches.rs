use crate::db::auth::auth::AuthUser;

use super::cache_manager::CacheManager;

lazy_static::lazy_static! {
    pub static ref USER_CACHE: CacheManager<i64, AuthUser> = CacheManager::new(500);
}