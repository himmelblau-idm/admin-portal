use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;
use zbus::Connection;

const BROKER_DEST: &str = "com.microsoft.identity.broker1";
const BROKER_PATH: &str = "/com/microsoft/identity/broker1";
const BROKER_IFACE: &str = "com.microsoft.identity.Broker1";
const PROTOCOL_VERSION: &str = "0.0";
const CLIENT_ID: &str = "d7b530a4-7680-4c23-a8bf-c52c121d2e87";

// ── D-Bus account/token types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerAccount {
    pub home_account_id: String,
    pub local_account_id: String,
    pub environment: String,
    pub realm: String,
    pub username: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub given_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetAccountsResponse {
    accounts: Vec<BrokerAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerTokenResponse {
    pub access_token: String,
    pub id_token: Option<String>,
    pub expires_on: Option<i64>,
    pub granted_scopes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcquireTokenResult {
    broker_token_response: Option<BrokerTokenResponse>,
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn correlation_id() -> String {
    Uuid::new_v4().to_string()
}

async fn broker_connection() -> Result<Connection, String> {
    Connection::session()
        .await
        .map_err(|e| format!("D-Bus session connection failed: {e}"))
}

async fn call_broker(
    conn: &Connection,
    method: &str,
    request_json: &str,
    call_timeout: Duration,
) -> Result<String, String> {
    let proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
        .destination(BROKER_DEST)
        .map_err(|e| format!("Invalid destination: {e}"))?
        .path(BROKER_PATH)
        .map_err(|e| format!("Invalid path: {e}"))?
        .interface(BROKER_IFACE)
        .map_err(|e| format!("Invalid interface: {e}"))?
        .cache_properties(zbus::proxy::CacheProperties::No)
        .build()
        .await
        .map_err(|e| format!("Proxy creation failed: {e}"))?;

    let correlation = correlation_id();
    let body = (PROTOCOL_VERSION, correlation.as_str(), request_json);
    let fut = proxy.call::<_, _, String>(method, &body);

    timeout(call_timeout, fut)
        .await
        .map_err(|_| format!("{method} timed out after {}s", call_timeout.as_secs()))?
        .map_err(|e| format!("{method} D-Bus call failed: {e}"))
        .and_then(|result| {
            if result.trim().is_empty() {
                Err(format!("{method} returned empty response (no cached token or broker unavailable)"))
            } else {
                Ok(result)
            }
        })
}

// ── Public broker functions ────────────────────────────────────────────────────

pub async fn broker_get_accounts() -> Result<Vec<BrokerAccount>, String> {
    let conn = broker_connection().await?;
    let request = serde_json::json!({
        "clientId": CLIENT_ID
    });
    let result = call_broker(&conn, "getAccounts", &request.to_string(), Duration::from_secs(15)).await?;
    let parsed: GetAccountsResponse =
        serde_json::from_str(&result).map_err(|e| format!("Failed to parse accounts: {e}"))?;
    Ok(parsed.accounts)
}

pub async fn broker_acquire_silent(account: &BrokerAccount) -> Result<BrokerTokenResponse, String> {
    let conn = broker_connection().await?;
    // TokenReq format: account at root + authParameters.requestedScopes (required)
    let request = serde_json::json!({
        "account": account,
        "authParameters": {
            "requestedScopes": ["https://graph.microsoft.com/.default"],
            "clientId": CLIENT_ID
        }
    });
    let result = call_broker(&conn, "acquireTokenSilently", &request.to_string(), Duration::from_secs(20)).await?;
    let parsed: AcquireTokenResult =
        serde_json::from_str(&result).map_err(|e| format!("Failed to parse token response: {e}"))?;
    parsed
        .broker_token_response
        .ok_or_else(|| "No token in silent response".to_string())
}

pub async fn broker_acquire_interactive() -> Result<(BrokerAccount, BrokerTokenResponse), String> {
    let conn = broker_connection().await?;

    // Get the cached account so we can pass username to the session broker
    // (it uses it to identify the PAM user for Pinentry auth)
    let accounts_request = serde_json::json!({ "clientId": CLIENT_ID });
    let accounts_result = call_broker(&conn, "getAccounts", &accounts_request.to_string(), Duration::from_secs(15)).await?;
    let accounts_parsed: GetAccountsResponse = serde_json::from_str(&accounts_result)
        .map_err(|e| format!("Failed to parse accounts: {e}"))?;
    let accounts = accounts_parsed.accounts;

    // TokenReq format: account at root + authParameters.requestedScopes (required).
    // The session broker uses account.username to invoke PAM / Pinentry, then
    // forwards this same JSON to the daemon's acquireTokenSilently.
    let request = if let Some(account) = accounts.first() {
        serde_json::json!({
            "account": account,
            "authParameters": {
                "requestedScopes": ["https://graph.microsoft.com/.default"],
                "clientId": CLIENT_ID
            }
        })
    } else {
        // No cached account — broker still needs a valid username; this will
        // likely fail, but give it a chance with an empty account object.
        serde_json::json!({
            "authParameters": {
                "requestedScopes": ["https://graph.microsoft.com/.default"],
                "clientId": CLIENT_ID
            }
        })
    };

    // Interactive auth can take several minutes (user must respond to Pinentry)
    let result = call_broker(&conn, "acquireTokenInteractively", &request.to_string(), Duration::from_secs(300)).await?;
    let parsed: AcquireTokenResult =
        serde_json::from_str(&result).map_err(|e| format!("Failed to parse token response: {e}"))?;
    let token = parsed
        .broker_token_response
        .ok_or_else(|| "No token in interactive response".to_string())?;

    // Re-fetch accounts to get the freshest account info after auth
    let updated_result = call_broker(&conn, "getAccounts", &accounts_request.to_string(), Duration::from_secs(15)).await.unwrap_or_default();
    let updated_accounts: Vec<BrokerAccount> = serde_json::from_str::<GetAccountsResponse>(&updated_result)
        .map(|r| r.accounts)
        .unwrap_or_default();

    let account = updated_accounts
        .into_iter()
        .next()
        .or_else(|| accounts.into_iter().next())
        .ok_or_else(|| "No account found after authentication".to_string())?;

    Ok((account, token))
}
