use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub uid: i64,
    pub email: String,
    pub owner: bool,
    pub email_verified: bool,
    pub username: String,
}
