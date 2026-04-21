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
pub fn CacheView(username: String) -> Element {
    // Cache clear
    let mut nss = use_signal(|| false);
    let mut mapped = use_signal(|| false);
    let mut full = use_signal(|| false);
    let mut clear_confirm = use_signal(|| false);
    let mut clear_loading = use_signal(|| false);
    let mut clear_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    // Enumerate
    let mut enum_client_id = use_signal(String::new);
    let mut enum_name = use_signal(|| username.clone());
    let mut enum_loading = use_signal(|| false);
    let mut enum_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run_clear = move |_| async move {
        if *full.read() && !*clear_confirm.read() {
            clear_confirm.set(true);
            return;
        }
        clear_confirm.set(false);
        clear_loading.set(true);
        #[derive(Serialize)]
        struct Args { nss: bool, mapped: bool, full: bool }
        let args = serde_wasm_bindgen::to_value(&Args {
            nss: *nss.read(),
            mapped: *mapped.read(),
            full: *full.read(),
        }).unwrap_or(JsValue::NULL);
        match invoke("aad_tool_cache_clear", args).await {
            Ok(js) => clear_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => clear_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        clear_loading.set(false);
    };

    let run_enumerate = move |_| async move {
        enum_loading.set(true);
        #[derive(Serialize)]
        struct Args { client_id: Option<String>, name: Option<String> }
        let cid = to_opt(enum_client_id.read().clone());
        let nm  = to_opt(enum_name.read().clone());
        let args = serde_wasm_bindgen::to_value(&Args { client_id: cid, name: nm })
            .unwrap_or(JsValue::NULL);
        match invoke("aad_tool_enumerate", args).await {
            Ok(js) => enum_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => enum_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        enum_loading.set(false);
    };

    let clear_el  = result_element(clear_result.read().clone());
    let enum_el   = result_element(enum_result.read().clone());
    let is_full   = *full.read();
    let confirming = *clear_confirm.read();

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "Cache" }
                p { "Clear the himmelblaud resolver cache or enumerate rfc2307 users and groups." }
            }

            // ── Cache Clear ────────────────────────────────────────────────
            div { class: "form-section",
                h3 { "Cache Clear" }
                p { class: "section-desc",
                    "Marks cached user/group entries as stale. Requires elevated privileges (pkexec)."
                }
                div { class: "checkbox-group",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: *nss.read(),
                            onchange: move |e| nss.set(e.value() == "true"),
                        }
                        " Only clear NSS cache"
                    }
                    label {
                        input {
                            r#type: "checkbox",
                            checked: *mapped.read(),
                            onchange: move |e| mapped.set(e.value() == "true"),
                        }
                        " Only clear mapped-name cache"
                    }
                    label {
                        input {
                            r#type: "checkbox",
                            checked: *full.read(),
                            onchange: move |e| full.set(e.value() == "true"),
                        }
                        " Full wipe — also unjoins host from Entra ID (irreversible!)"
                    }
                }
                if is_full {
                    div { class: "confirm-warning",
                        "⚠ Full wipe will unjoin this host from Entra ID. This cannot be undone."
                    }
                }
                if confirming {
                    div { class: "confirm-row",
                        button {
                            class: "run-btn danger-btn",
                            onclick: run_clear,
                            "Confirm Full Wipe"
                        }
                        button {
                            class: "cancel-btn",
                            onclick: move |_| clear_confirm.set(false),
                            "Cancel"
                        }
                    }
                } else {
                    button {
                        class: if is_full { "run-btn danger-btn" } else { "run-btn" },
                        disabled: *clear_loading.read(),
                        onclick: run_clear,
                        if *clear_loading.read() { "Clearing…" } else { "Clear Cache" }
                    }
                }
                {clear_el}
            }

            // ── Enumerate ──────────────────────────────────────────────────
            div { class: "form-section",
                h3 { "Enumerate" }
                p { class: "section-desc",
                    "Enumerate all users/groups with rfc2307 attributes and cache them locally."
                }
                div { class: "form-group",
                    label { r#for: "enum-cid", "Client ID (optional)" }
                    input {
                        id: "enum-cid",
                        r#type: "text",
                        placeholder: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
                        value: "{enum_client_id}",
                        oninput: move |e| enum_client_id.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "enum-name", "Account name (optional)" }
                    input {
                        id: "enum-name",
                        r#type: "text",
                        placeholder: "user@domain.com",
                        value: "{enum_name}",
                        oninput: move |e| enum_name.set(e.value()),
                    }
                }
                button {
                    class: "run-btn",
                    disabled: *enum_loading.read(),
                    onclick: run_enumerate,
                    if *enum_loading.read() { "Enumerating…" } else { "Enumerate" }
                }
                {enum_el}
            }
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
