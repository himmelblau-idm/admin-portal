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
pub fn LoginPage(checking: bool, on_login: EventHandler<TokenInfo>) -> Element {
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

    // Button is disabled while silent auth is in progress or interactive is running
    let busy = checking || *loading.read();

    rsx! {
        div {
            class: "login-wrapper",
            div {
                class: "login-card",
                div { class: "ms-logo-area",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "60",
                        height: "60",
                        view_box: "0 0 21 21",
                        rect { x: "1", y: "1", width: "9", height: "9", fill: "#f25022" }
                        rect { x: "11", y: "1", width: "9", height: "9", fill: "#7fba00" }
                        rect { x: "1", y: "11", width: "9", height: "9", fill: "#00a4ef" }
                        rect { x: "11", y: "11", width: "9", height: "9", fill: "#ffb900" }
                    }
                }
                h1 { class: "login-title", "Himmelblau Admin Portal" }
                p { class: "login-subtitle",
                    "Sign in with your Microsoft Entra ID account to continue"
                }

                if !error.read().is_empty() {
                    p { class: "error-msg", "{error}" }
                }

                button {
                    class: "ms-signin-btn",
                    disabled: busy,
                    onclick: sign_in,
                    if busy {
                        span { class: "btn-spinner" }
                        span {
                            if checking { "Checking saved session…" } else { "Signing in…" }
                        }
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

                if *loading.read() {
                    p { class: "pin-hint",
                        "A PIN dialog may appear — enter your Windows Hello PIN to continue."
                    }
                }

                p { class: "login-hint",
                    "Authentication is handled securely via the Himmelblau identity broker."
                }
            }
        }
    }
}

