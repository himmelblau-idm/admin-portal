#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::dashboard::Dashboard;
use crate::login::LoginPage;

static CSS: Asset = asset!("/assets/styles.css");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub username: String,
    pub name: String,
    pub access_token: String,
    pub expires_on: Option<i64>,
}

#[derive(Clone, Serialize, Deserialize)]
struct BrokerAccount {
    #[serde(rename = "homeAccountId")]
    pub home_account_id: String,
    #[serde(rename = "localAccountId")]
    pub local_account_id: String,
    pub environment: String,
    pub realm: String,
    pub username: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "givenName", default)]
    pub given_name: String,
}

#[derive(Clone, PartialEq)]
enum AppView {
    // Login page visible; checking=true while silent auth is in progress
    Login { checking: bool },
    Dashboard { name: String, username: String, access_token: String },
}

pub fn App() -> Element {
    let mut view = use_signal(|| AppView::Login { checking: true });

    // On mount: attempt silent auth in the background while Login page is shown.
    // If it succeeds, jump straight to Dashboard. If it fails, flip checking→false
    // so the login button becomes clickable.
    use_effect(move || {
        spawn(async move {
            match try_silent_auth().await {
                Some(info) => view.set(AppView::Dashboard {
                    name: info.name,
                    username: info.username,
                    access_token: info.access_token,
                }),
                None => view.set(AppView::Login { checking: false }),
            }
        });
    });

    let content = match view.read().clone() {
        AppView::Login { checking } => rsx! {
            LoginPage {
                checking,
                on_login: move |info: TokenInfo| view.set(AppView::Dashboard {
                    name: info.name,
                    username: info.username,
                    access_token: info.access_token,
                }),
            }
        },
        AppView::Dashboard { name, username, access_token } => rsx! {
            Dashboard {
                name,
                username,
                access_token,
                on_logout: move |_: ()| view.set(AppView::Login { checking: false }),
            }
        },
    };

    rsx! {
        link { rel: "stylesheet", href: CSS }
        {content}
    }
}

async fn try_silent_auth() -> Option<TokenInfo> {
    let empty = js_sys::Object::new();
    let accounts_js = invoke("get_accounts", empty.into()).await.ok()?;
    let accounts: Vec<BrokerAccount> =
        serde_wasm_bindgen::from_value(accounts_js).ok()?;
    let account = accounts.into_iter().next()?;

    let wrapper = js_sys::Object::new();
    let account_val = serde_wasm_bindgen::to_value(&account).ok()?;
    js_sys::Reflect::set(&wrapper, &"account".into(), &account_val).ok()?;
    let token_js = invoke("acquire_token_silent", wrapper.into()).await.ok()?;
    let info: TokenInfo = serde_wasm_bindgen::from_value(token_js).ok()?;
    Some(info)
}
