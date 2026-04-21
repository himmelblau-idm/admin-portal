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
pub fn AuthTestView(username: String) -> Element {
    let mut name = use_signal(|| username.clone());
    let mut password = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run = move |_| async move {
        let n = name.read().trim().to_string();
        if n.is_empty() {
            result.set(Some(Err("Account name is required".into())));
            return;
        }
        let p = password.read().clone();
        if p.is_empty() {
            result.set(Some(Err("Password or PIN is required".into())));
            return;
        }
        loading.set(true);
        #[derive(Serialize)]
        struct Args { name: String, password: String }
        let args = serde_wasm_bindgen::to_value(&Args { name: n, password: p })
            .unwrap_or(JsValue::NULL);
        match invoke("aad_tool_auth_test", args).await {
            Ok(js) => result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        loading.set(false);
    };

    let result_el = result_element(result.read().clone());

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "Auth Test" }
                p {
                    "Test authentication of a user via the himmelblaud resolver PAM channel. "
                    "This verifies that himmelblaud correctly processes and validates authentications."
                }
            }
            div { class: "form-section",
                div { class: "form-group",
                    label { r#for: "at-name", "Account (UPN or username)" }
                    input {
                        id: "at-name",
                        r#type: "text",
                        placeholder: "user@domain.com",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "at-pass", "Password or PIN" }
                    input {
                        id: "at-pass",
                        r#type: "password",
                        placeholder: "Entra ID password or Windows Hello PIN",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                }
                p { class: "field-hint",
                    "Your credential is sent directly to himmelblaud and is never stored."
                }
                button {
                    class: "run-btn",
                    disabled: *loading.read(),
                    onclick: run,
                    if *loading.read() { "Testing…" } else { "Run Auth Test" }
                }
            }
            {result_el}
        }
    }
}

fn result_element(r: Option<Result<String, String>>) -> Element {
    match r {
        Some(Ok(s))  => rsx! { pre { class: "result-box result-success", "{s}" } },
        Some(Err(e)) => rsx! { pre { class: "result-box result-error",   "{e}" } },
        None         => rsx! {},
    }
}
