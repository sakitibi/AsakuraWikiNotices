#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue};
use tauri::Manager;
use std::sync::Mutex;
use std::io::{self, Write};

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
    user_metadata: Option<serde_json::Value>,
}

// Supabaseの app_links テーブルから返ってくるデータ構造
#[derive(Deserialize, Debug, Clone)]
struct AppLinkRow {
    access_token: String,
    refresh_token: String,
    created_at: String,
}

fn log_info(msg: &str) {
    println!("{}", msg);
    let _ = io::stdout().flush();
}

// 💡 改行コード(\r, \n)、クォーテーション、空白、末尾のスラッシュを確実に完全に除去する強化版
fn get_env_var(env_content: &str, key: &str) -> Option<String> {
    for line in env_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                let cleaned_val = v.trim()
                    .trim_matches(|c| c == '"' || c == '\'' || c == '\r' || c == '\n')
                    .trim_end_matches('/'); // 末尾のスラッシュも自動で削る
                return Some(cleaned_val.to_string());
            }
        }
    }
    None
}

#[tauri::command]
async fn exchange_code_for_session(
    code: String,
    session_state: tauri::State<'_, SupabaseSession>,
) -> Result<UserResponse, String> {
    log_info(&format!("[Tauri] exchange_code_for_session: コード {} の検証を開始", code));

    let env_content = include_str!("../../../.env.local");
    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL")
        .ok_or_else(|| "SUPABASE_URLがありません".to_string())?;
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY")
        .ok_or_else(|| "SUPABASE_ANON_KEYがありません".to_string())?;

    let select_url = format!("{}/rest/v1/app_links?code=eq.{}&select=*", supabase_url_base, code);
    log_info(&format!("[DEBUG_URL] 通信先URL: {}", select_url));

    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| format!("API KEY設定エラー: {}", e))?);

    let client = reqwest::Client::new();
    
    let res = client.get(&select_url).headers(headers.clone()).send().await.map_err(|e| {
        log_info(&format!("[Tauri] 致命的な通信エラー (Supabaseに届く前に失敗): {}", e));
        format!("通信の送信自体に失敗しました: {}", e)
    })?;

    if !res.status().is_success() {
        let status_code = res.status();
        let err_text = res.text().await.unwrap_or_default();
        log_info(&format!("[Tauri] Supabase応答エラー ({}) - {}", status_code, err_text));
        return Err(format!("Supabaseがエラーを返しました ({}): {}", status_code, err_text));
    }

    let rows = res.json::<Vec<AppLinkRow>>().await.map_err(|e| format!("パースエラー: {}", e))?;

    if rows.is_empty() {
        log_info("[Tauri] コードが一致しない、または30秒以上経過しています。");
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
    let auth_res = client.get(&auth_url).headers(auth_headers).send().await.map_err(|e| format!("ユーザー検証通信失敗: {}", e))?;

    if !auth_res.status().is_success() {
        return Err("取得したセッションが無効です。".to_string());
    }

    let user_data = auth_res.json::<UserResponse>().await.map_err(|e| e.to_string())?;
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
    log_info("[Tauri] clear_supabase_session: セッションを破棄します。");
    *session_state.access_token.lock().unwrap() = None;
    *session_state.refresh_token.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
async fn verify_supabase_session(session_state: tauri::State<'_, SupabaseSession>) -> Result<UserResponse, String> {
    log_info("[Tauri] verify_supabase_session: 開始");
    let access_token = {
        let token_guard = session_state.access_token.lock().unwrap();
        token_guard.as_ref().ok_or_else(|| {
            log_info("[Tauri] verify_supabase_session: エラー - セッションがありません");
            "セッションがありません".to_string()
        })?.clone()
    };

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
    let res = client.get(&auth_url).headers(headers).send().await.map_err(|e| {
        log_info(&format!("[Tauri] verify_supabase_session: 通信エラー - {}", e));
        e.to_string()
    })?;

    if !res.status().is_success() {
        log_info(&format!("[Tauri] verify_supabase_session: 認証失敗 - ステータス: {}", res.status()));
        return Err("無効なセッションです".to_string());
    }

    let user_data = res.json::<UserResponse>().await.map_err(|e| {
        log_info(&format!("[Tauri] verify_supabase_session: パースエラー - {}", e));
        format!("ユーザー情報の解析失敗: {}", e)
    })?;

    log_info(&format!("[Tauri] verify_supabase_session: 認証成功 (Email: {:?})", user_data.email));
    Ok(user_data)
}

#[tauri::command]
async fn exchange_code_for_session(
    code: String,
    session_state: tauri::State<'_, SupabaseSession>,
) -> Result<UserResponse, String> {
    log_info(&format!("[Tauri] exchange_code_for_session: コード {} の検証を開始", code));

    let env_content = include_str!("../../../.env.local");
    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL")
        .ok_or_else(|| "SUPABASE_URLがありません".to_string())?;
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY")
        .ok_or_else(|| "SUPABASE_ANON_KEYがありません".to_string())?;

    // pg_cronのタイムラグをカバーするため、作成から30秒以内のデータのみを照合
    let select_url = format!(
        "{}/rest/v1/app_links?code=eq.{}&created_at=gt.now()-interval '30 seconds'&select=*", 
        supabase_url_base, code
    );

    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);

    let client = reqwest::Client::new();
    let res = client.get(&select_url).headers(headers.clone()).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        log_info(&format!("[Tauri] exchange_code_for_session: Supabaseエラー - ステータス {}", res.status()));
        return Err("Supabaseとの通信に失敗しました。".to_string());
    }

    let rows = res.json::<Vec<AppLinkRow>>().await.map_err(|e| format!("パースエラー: {}", e))?;

    if rows.is_empty() {
        log_info("[Tauri] exchange_code_for_session: コードが一致しない、または30秒以上経過しています。");
        return Err("認証コードが正しくないか、有効期限（30秒）が切れています。".to_string());
    }

    let target_row = &rows[0];

    // メモリ上の状態管理にトークンをセット
    *session_state.access_token.lock().unwrap() = Some(target_row.access_token.clone());
    *session_state.refresh_token.lock().unwrap() = Some(target_row.refresh_token.clone());

    // 使用済みのコードをテーブルから即時削除（使い回し防止）
    let delete_url = format!("{}/rest/v1/app_links?code=eq.{}", supabase_url_base, code);
    let _ = client.delete(&delete_url).headers(headers.clone()).send().await;

    // 取得したトークンでユーザー情報を検証
    drop(headers);
    let mut auth_headers = HeaderMap::new();
    auth_headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);
    auth_headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", target_row.access_token)).map_err(|e| e.to_string())?);

    let auth_url = format!("{}/auth/v1/user", supabase_url_base);
    let auth_res = client.get(&auth_url).headers(auth_headers).send().await.map_err(|e| e.to_string())?;

    if !auth_res.status().is_success() {
        log_info("[Tauri] exchange_code_for_session: トークン検証失敗");
        return Err("取得したセッションが無効です。".to_string());
    }

    let user_data = auth_res.json::<UserResponse>().await.map_err(|e| e.to_string())?;
    log_info(&format!("[Tauri] exchange_code_for_session: 認証成功 (Email: {:?})", user_data.email));

    Ok(user_data)
}

#[tauri::command]
async fn get_notices_from_supabase(session_state: tauri::State<'_, SupabaseSession>,) -> Result<Vec<Notice>, String> {
    log_info("[Tauri] get_notices_from_supabase: 開始");
    let env_content = include_str!("../../../.env.local");

    let supabase_url_base = get_env_var(env_content, "SUPABASE_URL")
        .ok_or_else(|| "環境変数 'SUPABASE_URL' が見つかりません。".to_string())?;
    
    let supabase_anon_key = get_env_var(env_content, "SUPABASE_ANON_KEY")
        .ok_or_else(|| "環境変数 'SUPABASE_ANON_KEY' が見つかりません。".to_string())?;

    let supabase_url = format!("{}/rest/v1/notices?select=*&order=created_at.desc", supabase_url_base);

    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(&supabase_anon_key).map_err(|e| e.to_string())?);

    let maybe_token = {
        let token_guard = session_state.access_token.lock().unwrap();
        token_guard.clone()
    };

    if let Some(access_token) = maybe_token {
        log_info("[Tauri] get_notices_from_supabase: 認証ヘッダーを付与します。");
        headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", access_token)).map_err(|e| e.to_string())?);
    } else {
        log_info("[Tauri] get_notices_from_supabase: 未ログイン（ゲスト状態）でリクエストします。");
    }

    let client = reqwest::Client::new();
    let res = client
        .get(&supabase_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| {
            log_info(&format!("[Tauri] get_notices_from_supabase: 通信エラー - {}", e));
            format!("通信エラー: {}", e)
        })?;

    if !res.status().is_success() {
        let status_code = res.status();
        let err_text = res.text().await.unwrap_or_default();
        log_info(&format!("[Tauri] get_notices_from_supabase: Supabaseエラー ({}) - {}", status_code, err_text));
        return Err(format!("Supabaseエラー ({}): {}", status_code, err_text));
    }

    let notices = res
        .json::<Vec<Notice>>()
        .await
        .map_err(|e| {
            log_info(&format!("[Tauri] get_notices_from_supabase: パースエラー - {}", e));
            format!("パースエラー: {}", e)
        })?;

    log_info(&format!("[Tauri] get_notices_from_supabase: 取得成功 (件数: {})", notices.len()));
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
                    log_info(&format!("[Tauri] SingleInstance: ディープリンクイベントを発行します -> {}", arg));
                    let _ = app.emit_all(DEEP_LINK_EVENT, arg);
                }
            }
        }))
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            for arg in args {
                if arg.starts_with("asakurawiki://") {
                    log_info(&format!("[Tauri] Setup: 起動引数からディープリンクを検知しました -> {}", arg));
                    let app_handle = app.handle();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        log_info("[Tauri] Setup: 1秒待機後、ディープリンクイベントを発行します。");
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
            verify_supabase_session,
            exchange_code_for_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}