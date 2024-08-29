use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref SECRETS: HashMap<String, String> = {
        let contents = std::fs::read_to_string("Secrets.toml").unwrap();
        let data: toml::Value = contents.parse().unwrap();
        let mut secrets = HashMap::new();
        secrets.insert("SECRET_KEY".to_string(), data["SECRET_KEY"].as_str().unwrap().to_string());
        secrets.insert("DB_NAME".to_string(), data["DB_NAME"].as_str().unwrap().to_string());
        secrets.insert("DB_PW".to_string(), data["DB_PW"].as_str().unwrap().to_string());
        secrets.insert("DB_PORT".to_string(), data["DB_PORT"].as_str().unwrap().to_string());
        secrets.insert("NO_REPLY_EMAIL".to_string(), data["NO_REPLY_EMAIL"].as_str().unwrap().to_string());
        secrets.insert("SMTP_USERNAME".to_string(), data["SMTP_USERNAME"].as_str().unwrap().to_string());
        secrets.insert("SMTP_PASSWORD".to_string(), data["SMTP_PASSWORD"].as_str().unwrap().to_string());
        secrets.insert("SMTP_RELAY".to_string(), data["SMTP_RELAY"].as_str().unwrap().to_string());
        secrets.insert("REPO".to_string(), data["REPO"].as_str().unwrap().to_string());
        secrets.insert("OWNER".to_string(), data["OWNER"].as_str().unwrap().to_string());
        secrets
    };
}