use std::fs::{self, File};
use std::path::PathBuf;
use crate::models::{SavedSessionJson, TokenUser, UserResponse, TOKEN_FILE_NAME};

pub fn get_token_file_path() -> Option<PathBuf> {
    dirs::home_dir().map(|mut path| {
        path.push(TOKEN_FILE_NAME);
        path
    })
}

pub fn save_session_to_file(access_token: &str, refresh_token: &str, user: &UserResponse) -> Result<(), String> {
    let path = get_token_file_path().ok_or_else(|| "ホームディレクトリが見つかりません".to_string())?;
    
    let session_data = SavedSessionJson {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.to_string(),
        token_type: "bearer".to_string(),
        expires_in: 3600,
        user: TokenUser {
            id: user.id.clone(),
            email: user.email.clone(),
        },
    };

    let file = File::create(&path).map_err(|e| format!("ファイル作成失敗: {}", e))?;
    serde_json::to_writer_pretty(file, &session_data).map_err(|e| format!("JSON書き込み失敗: {}", e))?;
    println!("[Tauri] セッションをファイルに保存しました: {:?}", path);
    Ok(())
}

pub fn load_session_from_file() -> Option<SavedSessionJson> {
    let path = get_token_file_path()?;
    if !path.exists() {
        return None;
    }
    let file = File::open(path).ok()?;
    serde_json::from_reader(file).ok()
}

pub fn delete_token_file() {
    if let Some(path) = get_token_file_path() {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }
}

pub fn get_env_var(env_content: &str, key: &str) -> Option<String> {
    for line in env_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                let cleaned_val = v.trim()
                    .trim_matches(|c| c == '"' || c == '\'' || c == '\r' || c == '\n')
                    .trim_end_matches('/');
                return Some(cleaned_val.to_string());
            }
        }
    }
    None
}