mod broker;

use broker::{BrokerAccount, BrokerTokenResponse, broker_acquire_interactive, broker_acquire_silent, broker_get_accounts};
use serde::Serialize;

#[derive(Serialize)]
pub struct TokenInfo {
    pub username: String,
    pub name: String,
    pub access_token: String,
    pub expires_on: Option<i64>,
}

fn token_info(account: &BrokerAccount, token: BrokerTokenResponse) -> TokenInfo {
    let display_name = if !account.name.is_empty() {
        account.name.clone()
    } else if !account.given_name.is_empty() {
        account.given_name.clone()
    } else {
        account.username.clone()
    };
    TokenInfo {
        username: account.username.clone(),
        name: display_name,
        access_token: token.access_token,
        expires_on: token.expires_on,
    }
}

#[tauri::command]
async fn get_accounts() -> Result<Vec<BrokerAccount>, String> {
    broker_get_accounts().await
}

#[tauri::command]
async fn acquire_token_silent(account: BrokerAccount) -> Result<TokenInfo, String> {
    let token = broker_acquire_silent(&account).await?;
    Ok(token_info(&account, token))
}

#[tauri::command]
async fn acquire_token_interactive() -> Result<TokenInfo, String> {
    let (account, token) = broker_acquire_interactive().await?;
    Ok(token_info(&account, token))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            acquire_token_silent,
            acquire_token_interactive
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
