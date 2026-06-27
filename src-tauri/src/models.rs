use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub const DEEP_LINK_EVENT: &str = "deep-link-login";
pub const TOKEN_FILE_NAME: &str = ".askreditor_token.json";

pub struct SupabaseSession {
    pub access_token: Mutex<Option<String>>,
    pub refresh_token: Mutex<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notice {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenUser {
    pub id: String,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserResponse {
    pub id: String,
    pub email: Option<String>,
    pub user_metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedSessionJson {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: TokenUser,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AppLinkRow {
    pub access_token: String,
    pub refresh_token: String,
    pub created_at: String,
}