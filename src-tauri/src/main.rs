#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue};
use tauri::Manager;

const DEEP_LINK_EVENT: &str = "deep-link-login";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Notice {
    id: String,
    title: String,
    content: String,
    created_at: String,
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
async fn get_notices_from_supabase() -> Result<Vec<Notice>, String> {
    let env_content = include_str!("../../../.env.local");

    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL")
        .ok_or_else(|| "環境変数 'SUPABASE_URL' が見つかりません。".to_string())?;
    
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY")
        .ok_or_else(|| "環境変数 'SUPABASE_ANON_KEY' が見つかりません。".to_string())?;

    let supabase_url = format!("{}/rest/v1/notices?select=*&order=created_at.desc", supabase_url_base);

    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);
    headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", supabase_anon_key)).map_err(|e| e.to_string())?);

    let client = reqwest::Client::new();
    let res = client
        .get(&supabase_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("通信エラー: {}", e))?;

    let notices = res
        .json::<Vec<Notice>>()
        .await
        .map_err(|e| format!("パースエラー: {}", e))?;

    Ok(notices)
}

fn main() {
    tauri::Builder::default()
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
        .invoke_handler(tauri::generate_handler![get_notices_from_supabase])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}