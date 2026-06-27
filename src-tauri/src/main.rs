#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod auth;

use models::{SupabaseSession, UserResponse, AppLinkRow, Notice};
use auth::{get_env_var, save_session_to_file, load_session_from_file, delete_token_file};

use serde_json;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::Mutex;
use std::io::{self, Write};

fn log_info(msg: &str) {
    println!("{}", msg);
    let _ = io::stdout().flush();
}

#[tauri::command]
async fn exchange_code_for_session(
    code: String,
    session_state: tauri::State<'_, SupabaseSession>,
) -> Result<UserResponse, String> {
    log_info(&format!("[Tauri] exchange_code_for_session: コード {} の検証を開始", code));

    let env_content = include_str!("../../../.env.local");
    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL").ok_or("SUPABASE_URLがありません")?;
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY").ok_or("SUPABASE_ANON_KEYがありません")?;

    let select_url = format!("{}/rest/v1/app_links?code=eq.{}&select=*", supabase_url_base, code);
    log_info(&format!("[DEBUG_URL] 通信先URL: {}", select_url));

    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);

    let client = reqwest::Client::new();
    let res = client.get(&select_url).headers(headers.clone()).send().await.map_err(|e| format!("通信送信失敗: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Supabaseエラー: {}", res.status()));
    }

    let rows = res.json::<Vec<AppLinkRow>>().await.map_err(|e| format!("パースエラー: {}", e))?;
    if rows.is_empty() {
        return Err("認証コードが正しくないか、有効期限が切れています。".to_string());
    }

    let target_row = &rows[0];
    *session_state.access_token.lock().unwrap() = Some(target_row.access_token.clone());
    *session_state.refresh_token.lock().unwrap() = Some(target_row.refresh_token.clone());

    let delete_url = format!("{}/rest/v1/app_links?code=eq.{}", supabase_url_base, code);
    let _ = client.delete(&delete_url).headers(headers.clone()).send().await;

    drop(headers);
    let mut auth_headers = HeaderMap::new();
    auth_headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);
    auth_headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", target_row.access_token)).map_err(|e| e.to_string())?);

    let auth_url = format!("{}/auth/v1/user", supabase_url_base);
    let auth_res = client.get(&auth_url).headers(auth_headers).send().await.map_err(|e| e.to_string())?;

    if !auth_res.status().is_success() {
        return Err("取得したセッションが無効です。".to_string());
    }

    let user_data = auth_res.json::<UserResponse>().await.map_err(|e| e.to_string())?;
    let _ = save_session_to_file(&target_row.access_token, &target_row.refresh_token, &user_data);

    Ok(user_data)
}

#[tauri::command]
async fn set_supabase_session(
    access_token: String,
    refresh_token: String,
    session_state: tauri::State<'_, SupabaseSession>,
) -> Result<(), String> {
    log_info("[Tauri] set_supabase_session: セッションを設定します。");
    *session_state.access_token.lock().unwrap() = Some(access_token);
    *session_state.refresh_token.lock().unwrap() = Some(refresh_token);
    Ok(())
}

#[tauri::command]
async fn clear_supabase_session(session_state: tauri::State<'_, SupabaseSession>) -> Result<(), String> {
    log_info("[Tauri] clear_supabase_session: セッションを破棄し、ファイルを削除します。");
    *session_state.access_token.lock().unwrap() = None;
    *session_state.refresh_token.lock().unwrap() = None;
    delete_token_file();
    Ok(())
}

#[tauri::command]
async fn verify_supabase_session(session_state: tauri::State<'_, SupabaseSession>) -> Result<UserResponse, String> {
    log_info("[Tauri] verify_supabase_session: 開始");
    
    let mut access_token = session_state.access_token.lock().unwrap().clone();

    if access_token.is_none() {
        if let Some(saved) = load_session_from_file() {
            log_info("[Tauri] ファイルから保存済みセッションを自動検出しました。");
            *session_state.access_token.lock().unwrap() = Some(saved.access_token.clone());
            *session_state.refresh_token.lock().unwrap() = Some(saved.refresh_token.clone());
            access_token = Some(saved.access_token);
        }
    }

    let token = access_token.ok_or("セッションがありません")?;
    let env_content = include_str!("../../../.env.local");
    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL").ok_or("SUPABASE_URLがありません")?;
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY").ok_or("SUPABASE_ANON_KEYがありません")?;

    let auth_url = format!("{}/auth/v1/user", supabase_url_base);
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);
    headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| e.to_string())?);

    let client = reqwest::Client::new();
    let res = client.get(&auth_url).headers(headers).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        delete_token_file();
        return Err("無効なセッションです".to_string());
    }

    let user_data = res.json::<UserResponse>().await.map_err(|e| format!("解析失敗: {}", e))?;
    log_info(&format!("[Tauri] verify_supabase_session: 認証成功 (Email: {:?})", user_data.email));
    Ok(user_data)
}

#[tauri::command]
async fn get_notices_from_supabase(session_state: tauri::State<'_, SupabaseSession>,) -> Result<Vec<Notice>, String> {
    let env_content = include_str!("../../../.env.local");
    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL").ok_or("SUPABASE_URLがありません")?;
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY").ok_or("SUPABASE_ANON_KEYがありません")?;

    let supabase_url = format!("{}/rest/v1/notices?select=*&order=created_at.desc", supabase_url_base);
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);

    if let Some(access_token) = session_state.access_token.lock().unwrap().clone() {
        headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", access_token)).map_err(|e| e.to_string())?);
    }

    let client = reqwest::Client::new();
    let res = client.get(&supabase_url).headers(headers).send().await.map_err(|e| e.to_string())?;
    
    if !res.status().is_success() {
        return Err(format!("Supabaseエラー: {}", res.status()));
    }

    let notices = res.json::<Vec<Notice>>().await.map_err(|e| e.to_string())?;
    Ok(notices)
}

fn main() {
    tauri::Builder::default()
        .manage(SupabaseSession {
            access_token: Mutex::new(None),
            refresh_token: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_notices_from_supabase,
            set_supabase_session,
            clear_supabase_session,
            verify_supabase_session,
            exchange_code_for_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}