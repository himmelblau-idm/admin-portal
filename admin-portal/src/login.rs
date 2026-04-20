#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

use crate::app::TokenInfo;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn LoginPage(on_login: EventHandler<TokenInfo>) -> Element {
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| String::new());

    let sign_in = move |_| async move {
        loading.set(true);
        error.set(String::new());

        // acquire_token_interactive takes no params — pass empty object
        let empty = js_sys::Object::new();
        match invoke("acquire_token_interactive", empty.into()).await {
            Ok(js_val) => {
                match serde_wasm_bindgen::from_value::<TokenInfo>(js_val) {
                    Ok(info) => on_login.call(info),
                    Err(e) => {
                        error.set(format!("Failed to parse token response: {e}"));
                        loading.set(false);
                    }
                }
            }
            Err(e) => {
                let msg = e
                    .as_string()
                    .unwrap_or_else(|| "Authentication failed. Please try again.".to_string());
                error.set(msg);
                loading.set(false);
            }
        }
    };

    rsx! {
        div {
            class: "login-wrapper",
            div {
                class: "login-card",
                div { class: "ms-logo-area",
                    span { class: "ms-logo", "⬛" }
                }
                h1 { class: "login-title", "Admin Portal" }
                p { class: "login-subtitle",
                    "Sign in with your Microsoft Entra ID account"
                }

                if !error.read().is_empty() {
                    p { class: "error-msg", "{error}" }
                }

                button {
                    class: "ms-signin-btn",
                    disabled: *loading.read(),
                    onclick: sign_in,
                    if *loading.read() {
                        span { class: "btn-spinner" }
                        span { "Signing in…" }
                    } else {
                        span { class: "ms-icon",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "21",
                                height: "21",
                                view_box: "0 0 21 21",
                                rect { x: "1", y: "1", width: "9", height: "9", fill: "#f25022" }
                                rect { x: "11", y: "1", width: "9", height: "9", fill: "#7fba00" }
                                rect { x: "1", y: "11", width: "9", height: "9", fill: "#00a4ef" }
                                rect { x: "11", y: "11", width: "9", height: "9", fill: "#ffb900" }
                            }
                        }
                        span { "Sign in with Microsoft" }
                    }
                }

                p { class: "login-hint",
                    "Authentication is handled securely via the Himmelblau identity broker."
                }
            }
        }
    }
}

