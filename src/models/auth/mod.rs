use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username_or_email: String,
    password: String,
}
