#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue};
use tauri::Manager;
use std::sync::Mutex;

const DEEP_LINK_EVENT: &str = "deep-link-login";

struct SupabaseSession {
    access_token: Mutex<Option<String>>,
    refresh_token: Mutex<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Notice {
    id: String,
    title: String,
    content: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserResponse {
    id: String,
    email: Option<String>,
    // メタデータ（ユーザー名などが入る場所）
    user_metadata: Option<serde_json::Value>,
}

fn get_env_var(env_content: &str, key: &str) -> Option<String> {
    for line in env_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().trim_matches(|c| c == '"' || c == '\'').to_string());
            }
        }
    }
    None
}

#[tauri::command]
async fn set_supabase_session(
    accessToken: String,
    refreshToken: String,
    session_state: tauri::State<'_, SupabaseSession>,
) -> Result<(), String> {
    *session_state.access_token.lock().unwrap() = Some(accessToken);
    *session_state.refresh_token.lock().unwrap() = Some(refreshToken);
    Ok(())
}

#[tauri::command]
async fn clear_supabase_session(session_state: tauri::State<'_, SupabaseSession>) -> Result<(), String> {
    *session_state.access_token.lock().unwrap() = None;
    *session_state.refresh_token.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
async fn verify_supabase_session(session_state: tauri::State<'_, SupabaseSession>) -> Result<UserResponse, String> {
    let token_guard = session_state.access_token.lock().unwrap();
    let access_token = token_guard.as_ref().ok_or_else(|| "セッションがありません".to_string())?;

    let env_content = include_str!("../../../.env.local");
    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL")
        .ok_or_else(|| "SUPABASE_URLがありません".to_string())?;
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY")
        .ok_or_else(|| "SUPABASE_ANON_KEYがありません".to_string())?;

    let auth_url = format!("{}/auth/v1/user", supabase_url_base);

    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);
    headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", access_token)).map_err(|e| e.to_string())?);

    let client = reqwest::Client::new();
    let res = client.get(&auth_url).headers(headers).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err("無効なセッションです".to_string());
    }

    // ユーザー情報をJSON構造体にパース
    let user_data = res.json::<UserResponse>().await.map_err(|e| format!("ユーザー情報の解析失敗: {}", e))?;
    Ok(user_data)
}

#[tauri::command]
async fn get_notices_from_supabase(session_state: tauri::State<'_, SupabaseSession>) -> Result<Vec<Notice>, String> {
    let env_content = include_str!("../../../.env.local");

    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL")
        .ok_or_else(|| "環境変数 'SUPABASE_URL' が見つかりません。".to_string())?;
    
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY")
        .ok_or_else(|| "環境変数 'SUPABASE_ANON_KEY' が見つかりません。".to_string())?;

    let supabase_url = format!("{}/rest/v1/notices?select=*&order=created_at.desc", supabase_url_base);

    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);

    let token_guard = session_state.access_token.lock().unwrap();
    if let Some(access_token) = token_guard.as_ref() {
        headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", access_token)).map_err(|e| e.to_string())?);
    }

    let client = reqwest::Client::new();
    let res = client
        .get(&supabase_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("通信エラー: {}", e))?;

    if !res.status().is_success() {
        let status_code = res.status();
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Supabaseエラー ({}): {}", status_code, err_text));
    }

    let notices = res
        .json::<Vec<Notice>>()
        .await
        .map_err(|e| format!("パースエラー: {}", e))?;

    Ok(notices)
}

fn main() {
    tauri::Builder::default()
        .manage(SupabaseSession {
            access_token: Mutex::new(None),
            refresh_token: Mutex::new(None),
        })
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            for arg in args {
                if arg.starts_with("asakurawiki://") {
                    let _ = app.emit_all(DEEP_LINK_EVENT, arg);
                }
            }
        }))
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            for arg in args {
                if arg.starts_with("asakurawiki://") {
                    let app_handle = app.handle();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        let _ = app_handle.emit_all(DEEP_LINK_EVENT, arg);
                    });
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_notices_from_supabase,
            set_supabase_session,
            clear_supabase_session,
            verify_supabase_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}