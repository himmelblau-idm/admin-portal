mod aad_tool;
mod broker;

use aad_tool::{run_aad_tool, run_aad_tool_as_root};
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

// ── System checks ─────────────────────────────────────────────────────────────

/// Returns true if the process is running with effective UID 0 (root).
/// Reads /proc/self/status which is always available on Linux.
#[tauri::command]
fn check_is_root() -> bool {
    effective_uid() == 0
}

fn effective_uid() -> u32 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                // "Uid:  <real>  <effective>  <saved>  <filesystem>"
                .and_then(|l| l.split_whitespace().nth(2))
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(1000) // assume non-root if unreadable
}

// ── Status ────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_status() -> Result<String, String> {
    run_aad_tool(vec!["status".into()]).await
}

#[tauri::command]
async fn aad_tool_tpm() -> Result<String, String> {
    run_aad_tool_as_root(vec!["tpm".into()]).await
}

#[tauri::command]
async fn aad_tool_version() -> Result<String, String> {
    run_aad_tool(vec!["version".into()]).await
}

// ── Cache ─────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_cache_clear(nss: bool, mapped: bool, full: bool) -> Result<String, String> {
    let mut args = vec!["cache-clear".to_string()];
    if nss { args.push("--nss".into()); }
    if mapped { args.push("--mapped".into()); }
    if full { args.push("--full".into()); }
    run_aad_tool_as_root(args).await
}

#[tauri::command]
async fn aad_tool_enumerate(
    client_id: Option<String>,
    name: Option<String>,
) -> Result<String, String> {
    let mut args = vec!["enumerate".to_string()];
    if let Some(cid) = client_id { args.extend(["--client-id".into(), cid]); }
    if let Some(n) = name { args.extend(["--name".into(), n]); }
    run_aad_tool_as_root(args).await
}

// ── Application ───────────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_application_list(
    client_id: String,
    name: Option<String>,
) -> Result<String, String> {
    let mut args = vec!["application".to_string(), "list".into(), "--client-id".into(), client_id];
    if let Some(n) = name { args.extend(["--name".into(), n]); }
    run_aad_tool(args).await
}

#[tauri::command]
async fn aad_tool_application_create(
    client_id: String,
    display_name: String,
    redirect_uris: Vec<String>,
    user_read_write: bool,
    group_read_write: bool,
    name: Option<String>,
) -> Result<String, String> {
    let mut args = vec![
        "application".to_string(), "create".into(),
        "--client-id".into(), client_id,
        "--display-name".into(), display_name,
    ];
    for uri in redirect_uris {
        args.extend(["--redirect-uri".into(), uri]);
    }
    if user_read_write { args.push("--user-read-write".into()); }
    if group_read_write { args.push("--group-read-write".into()); }
    if let Some(n) = name { args.extend(["--name".into(), n]); }
    run_aad_tool(args).await
}

#[tauri::command]
async fn aad_tool_application_list_schema(
    client_id: String,
    schema_app_object_id: String,
    name: Option<String>,
) -> Result<String, String> {
    let mut args = vec![
        "application".to_string(), "list-schema-extensions".into(),
        "--client-id".into(), client_id,
        "--schema-app-object-id".into(), schema_app_object_id,
    ];
    if let Some(n) = name { args.extend(["--name".into(), n]); }
    run_aad_tool(args).await
}

#[tauri::command]
async fn aad_tool_application_add_schema(
    client_id: String,
    schema_app_object_id: String,
    name: Option<String>,
) -> Result<String, String> {
    let mut args = vec![
        "application".to_string(), "add-schema-extensions".into(),
        "--client-id".into(), client_id,
        "--schema-app-object-id".into(), schema_app_object_id,
    ];
    if let Some(n) = name { args.extend(["--name".into(), n]); }
    run_aad_tool(args).await
}

// ── User ──────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_user_set_posix(
    schema_client_id: String,
    user_id: String,
    uid: Option<u32>,
    gid: Option<u32>,
    home: Option<String>,
    shell: Option<String>,
    gecos: Option<String>,
    name: Option<String>,
) -> Result<String, String> {
    let mut args = vec![
        "user".to_string(), "set-posix-attrs".into(),
        "--schema-client-id".into(), schema_client_id,
        "--user-id".into(), user_id,
    ];
    if let Some(v) = uid   { args.extend(["--uid".into(),   v.to_string()]); }
    if let Some(v) = gid   { args.extend(["--gid".into(),   v.to_string()]); }
    if let Some(v) = home  { args.extend(["--home".into(),  v]); }
    if let Some(v) = shell { args.extend(["--shell".into(), v]); }
    if let Some(v) = gecos { args.extend(["--gecos".into(), v]); }
    if let Some(n) = name  { args.extend(["--name".into(),  n]); }
    run_aad_tool(args).await
}

// ── Group ─────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_group_set_posix(
    schema_client_id: String,
    group_id: String,
    gid: u32,
    name: Option<String>,
) -> Result<String, String> {
    let mut args = vec![
        "group".to_string(), "set-posix-attrs".into(),
        "--schema-client-id".into(), schema_client_id,
        "--group-id".into(), group_id,
        "--gid".into(), gid.to_string(),
    ];
    if let Some(n) = name { args.extend(["--name".into(), n]); }
    run_aad_tool(args).await
}

// ── ID Map ────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_idmap_user_add(account_name: String, uid: u32, gid: u32) -> Result<String, String> {
    run_aad_tool_as_root(vec![
        "idmap".into(), "user-add".into(),
        "--name".into(), account_name,
        "--uid".into(), uid.to_string(),
        "--gid".into(), gid.to_string(),
    ]).await
}

#[tauri::command]
async fn aad_tool_idmap_group_add(object_id: String, gid: u32) -> Result<String, String> {
    run_aad_tool_as_root(vec![
        "idmap".into(), "group-add".into(),
        "--object_id".into(), object_id,
        "--gid".into(), gid.to_string(),
    ]).await
}

#[tauri::command]
async fn aad_tool_idmap_clear() -> Result<String, String> {
    run_aad_tool_as_root(vec!["idmap".into(), "clear".into()]).await
}

// ── Credentials ───────────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_cred_list(domain: String) -> Result<String, String> {
    run_aad_tool_as_root(vec!["cred".into(), "list".into(), "--domain".into(), domain]).await
}

#[tauri::command]
async fn aad_tool_cred_secret(
    client_id: String,
    domain: String,
    secret: String,
) -> Result<String, String> {
    run_aad_tool_as_root(vec![
        "cred".into(), "secret".into(),
        "--client-id".into(), client_id,
        "--domain".into(), domain,
        "--secret".into(), secret,
    ]).await
}

#[tauri::command]
async fn aad_tool_cred_cert(
    client_id: String,
    domain: String,
    valid_days: u32,
    cert_out: String,
) -> Result<String, String> {
    run_aad_tool_as_root(vec![
        "cred".into(), "cert".into(),
        "--client-id".into(), client_id,
        "--domain".into(), domain,
        "--valid-days".into(), valid_days.to_string(),
        "--cert-out".into(), cert_out,
    ]).await
}

#[tauri::command]
async fn aad_tool_cred_delete(
    domain: String,
    secret_only: bool,
    cert_only: bool,
) -> Result<String, String> {
    let mut args = vec!["cred".into(), "delete".into(), "--domain".into(), domain];
    if secret_only { args.push("--secret".into()); }
    if cert_only   { args.push("--cert".into()); }
    run_aad_tool_as_root(args).await
}

// ── PAM ───────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_configure_pam(
    really: bool,
    auth_file: Option<String>,
    account_file: Option<String>,
    session_file: Option<String>,
    password_file: Option<String>,
) -> Result<String, String> {
    let mut args = vec!["configure-pam".to_string()];
    if really { args.push("--really".into()); }
    if let Some(f) = auth_file     { args.extend(["--auth-file".into(),     f]); }
    if let Some(f) = account_file  { args.extend(["--account-file".into(),  f]); }
    if let Some(f) = session_file  { args.extend(["--session-file".into(),  f]); }
    if let Some(f) = password_file { args.extend(["--password-file".into(), f]); }
    run_aad_tool_as_root(args).await
}

// ── Offline Breakglass ────────────────────────────────────────────────────────

#[tauri::command]
async fn aad_tool_offline_breakglass(ttl: Option<String>) -> Result<String, String> {
    let mut args = vec!["offline-breakglass".to_string()];
    if let Some(t) = ttl { args.extend(["--ttl".into(), t]); }
    run_aad_tool_as_root(args).await
}

// ── App entry point ───────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // System checks
            check_is_root,
            // Broker / auth
            get_accounts,
            acquire_token_silent,
            acquire_token_interactive,
            // Status
            aad_tool_status,
            aad_tool_tpm,
            aad_tool_version,
            // Auth test
            // aad_tool_auth_test,
            // Cache
            aad_tool_cache_clear,
            aad_tool_enumerate,
            // Application
            aad_tool_application_list,
            aad_tool_application_create,
            aad_tool_application_list_schema,
            aad_tool_application_add_schema,
            // User / Group
            aad_tool_user_set_posix,
            aad_tool_group_set_posix,
            // ID Map
            aad_tool_idmap_user_add,
            aad_tool_idmap_group_add,
            aad_tool_idmap_clear,
            // Credentials
            aad_tool_cred_list,
            aad_tool_cred_secret,
            aad_tool_cred_cert,
            aad_tool_cred_delete,
            // PAM
            aad_tool_configure_pam,
            // Offline breakglass
            aad_tool_offline_breakglass,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
