#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn PamView() -> Element {
    let mut really = use_signal(|| false);
    let mut auth_file = use_signal(String::new);
    let mut account_file = use_signal(String::new);
    let mut session_file = use_signal(String::new);
    let mut password_file = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run = move |_| async move {
        loading.set(true);
        #[derive(Serialize)]
        struct Args {
            really: bool,
            auth_file: Option<String>,
            account_file: Option<String>,
            session_file: Option<String>,
            password_file: Option<String>,
        }
        let args = serde_wasm_bindgen::to_value(&Args {
            really: *really.read(),
            auth_file:    to_opt(auth_file.read().clone()),
            account_file: to_opt(account_file.read().clone()),
            session_file: to_opt(session_file.read().clone()),
            password_file: to_opt(password_file.read().clone()),
        }).unwrap_or(JsValue::NULL);
        match invoke("aad_tool_configure_pam", args).await {
            Ok(js) => result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        loading.set(false);
    };

    let result_el = result_element(result.read().clone());
    let is_really = *really.read();

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "PAM Configuration" }
                p {
                    "Configure PAM to use pam_himmelblau. "
                    "Requires elevated privileges (pkexec). "
                    "You must check \"Apply changes\" to actually modify the PAM files."
                }
            }
            div { class: "form-section",
                div { class: "form-group",
                    label { r#for: "pam-auth", "Auth PAM file (optional, uses default)" }
                    input {
                        id: "pam-auth",
                        r#type: "text",
                        placeholder: "/etc/pam.d/common-auth",
                        value: "{auth_file}",
                        oninput: move |e| auth_file.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "pam-account", "Account PAM file (optional)" }
                    input {
                        id: "pam-account",
                        r#type: "text",
                        placeholder: "/etc/pam.d/common-account",
                        value: "{account_file}",
                        oninput: move |e| account_file.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "pam-session", "Session PAM file (optional)" }
                    input {
                        id: "pam-session",
                        r#type: "text",
                        placeholder: "/etc/pam.d/common-session",
                        value: "{session_file}",
                        oninput: move |e| session_file.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "pam-password", "Password PAM file (optional)" }
                    input {
                        id: "pam-password",
                        r#type: "text",
                        placeholder: "/etc/pam.d/common-password",
                        value: "{password_file}",
                        oninput: move |e| password_file.set(e.value()),
                    }
                }
                label { class: "checkbox-label",
                    input {
                        r#type: "checkbox",
                        checked: is_really,
                        onchange: move |e| really.set(e.value() == "true"),
                    }
                    " Apply changes (without this, the command runs in dry-run mode)"
                }
                if is_really {
                    div { class: "confirm-warning",
                        "⚠ This will modify your PAM configuration files."
                    }
                }
                button {
                    class: "run-btn",
                    disabled: *loading.read(),
                    onclick: run,
                    if *loading.read() {
                        "Configuring…"
                    } else if is_really {
                        "Apply PAM Config"
                    } else {
                        "Dry Run (preview only)"
                    }
                }
            }
            {result_el}
        }
    }
}

fn to_opt(s: String) -> Option<String> {
    let s = s.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn result_element(r: Option<Result<String, String>>) -> Element {
    match r {
        Some(Ok(s))  => rsx! { pre { class: "result-box result-success", "{s}" } },
        Some(Err(e)) => rsx! { pre { class: "result-box result-error",   "{e}" } },
        None         => rsx! {},
    }
}
