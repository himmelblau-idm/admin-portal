#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone, PartialEq)]
enum CredsTab { List, Secret, Cert, Delete }

#[component]
pub fn CredsView() -> Element {
    let mut tab = use_signal(|| CredsTab::List);

    // Shared domain
    let mut domain = use_signal(String::new);

    // List
    let mut list_loading = use_signal(|| false);
    let mut list_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    // Secret
    let mut sec_client_id = use_signal(String::new);
    let mut sec_secret    = use_signal(String::new);
    let mut sec_loading   = use_signal(|| false);
    let mut sec_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    // Cert
    let mut cert_client_id  = use_signal(String::new);
    let mut cert_valid_days = use_signal(|| "365".to_string());
    let mut cert_out        = use_signal(|| "/tmp/himmelblau-cert.pem".to_string());
    let mut cert_loading    = use_signal(|| false);
    let mut cert_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    // Delete
    let mut del_secret_only  = use_signal(|| false);
    let mut del_cert_only    = use_signal(|| false);
    let mut del_confirm      = use_signal(|| false);
    let mut del_loading      = use_signal(|| false);
    let mut del_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run_list = move |_| async move {
        let d = domain.read().trim().to_string();
        if d.is_empty() { list_result.set(Some(Err("Domain is required".into()))); return; }
        list_loading.set(true);
        #[derive(Serialize)] struct Args { domain: String }
        let args = serde_wasm_bindgen::to_value(&Args { domain: d }).unwrap_or(JsValue::NULL);
        match invoke("aad_tool_cred_list", args).await {
            Ok(js) => list_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => list_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        list_loading.set(false);
    };

    let run_secret = move |_| async move {
        let d   = domain.read().trim().to_string();
        let cid = sec_client_id.read().trim().to_string();
        let sec = sec_secret.read().trim().to_string();
        if d.is_empty() || cid.is_empty() || sec.is_empty() {
            sec_result.set(Some(Err("Domain, Client ID, and Secret are all required".into())));
            return;
        }
        sec_loading.set(true);
        #[derive(Serialize)] struct Args { client_id: String, domain: String, secret: String }
        let args = serde_wasm_bindgen::to_value(&Args { client_id: cid, domain: d, secret: sec })
            .unwrap_or(JsValue::NULL);
        match invoke("aad_tool_cred_secret", args).await {
            Ok(js) => sec_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => sec_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        sec_loading.set(false);
    };

    let run_cert = move |_| async move {
        let d   = domain.read().trim().to_string();
        let cid = cert_client_id.read().trim().to_string();
        let days_str = cert_valid_days.read().trim().to_string();
        let out = cert_out.read().trim().to_string();
        if d.is_empty() || cid.is_empty() || days_str.is_empty() || out.is_empty() {
            cert_result.set(Some(Err("All fields are required".into())));
            return;
        }
        let valid_days: u32 = match days_str.parse() {
            Ok(v) => v,
            Err(_) => { cert_result.set(Some(Err("Valid days must be a positive integer".into()))); return; }
        };
        cert_loading.set(true);
        #[derive(Serialize)] struct Args { client_id: String, domain: String, valid_days: u32, cert_out: String }
        let args = serde_wasm_bindgen::to_value(&Args {
            client_id: cid, domain: d, valid_days, cert_out: out,
        }).unwrap_or(JsValue::NULL);
        match invoke("aad_tool_cred_cert", args).await {
            Ok(js) => cert_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => cert_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        cert_loading.set(false);
    };

    let run_delete = move |_| async move {
        let d = domain.read().trim().to_string();
        if d.is_empty() { del_result.set(Some(Err("Domain is required".into()))); return; }
        if !*del_confirm.read() { del_confirm.set(true); return; }
        del_confirm.set(false);
        del_loading.set(true);
        #[derive(Serialize)] struct Args { domain: String, secret_only: bool, cert_only: bool }
        let args = serde_wasm_bindgen::to_value(&Args {
            domain: d,
            secret_only: *del_secret_only.read(),
            cert_only:   *del_cert_only.read(),
        }).unwrap_or(JsValue::NULL);
        match invoke("aad_tool_cred_delete", args).await {
            Ok(js) => del_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => del_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        del_loading.set(false);
    };

    let active = tab.read().clone();

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "Credentials" }
                p {
                    "Manage confidential client credentials (secrets and certificates) for authenticating "
                    "to Entra ID. Requires elevated privileges (pkexec)."
                }
            }

            // Shared domain
            div { class: "form-group shared-field",
                label { r#for: "cred-domain", "Domain (shared across all tabs) *" }
                input {
                    id: "cred-domain",
                    r#type: "text",
                    placeholder: "yourdomain.onmicrosoft.com",
                    value: "{domain}",
                    oninput: move |e| domain.set(e.value()),
                }
            }

            div { class: "tab-bar",
                button {
                    class: if active == CredsTab::List { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(CredsTab::List),
                    "List"
                }
                button {
                    class: if active == CredsTab::Secret { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(CredsTab::Secret),
                    "Add Secret"
                }
                button {
                    class: if active == CredsTab::Cert { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(CredsTab::Cert),
                    "Add Certificate"
                }
                button {
                    class: if active == CredsTab::Delete { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(CredsTab::Delete),
                    "Delete"
                }
            }

            {match active {
                CredsTab::List => {
                    let el = result_element(list_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            p { class: "section-desc", "Check whether a client secret and/or certificate exists for this domain." }
                            button {
                                class: "run-btn",
                                disabled: *list_loading.read(),
                                onclick: run_list,
                                if *list_loading.read() { "Checking…" } else { "List Credentials" }
                            }
                            {el}
                        }
                    }
                }
                CredsTab::Secret => {
                    let el = result_element(sec_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            p { class: "section-desc",
                                "Store a client secret copied from the Entra ID portal (Certificates & secrets tab)."
                            }
                            div { class: "form-group",
                                label { r#for: "cs-cid", "Client ID *" }
                                input {
                                    id: "cs-cid",
                                    r#type: "text",
                                    placeholder: "Application (client) ID",
                                    value: "{sec_client_id}",
                                    oninput: move |e| sec_client_id.set(e.value()),
                                }
                            }
                            div { class: "form-group",
                                label { r#for: "cs-sec", "Client secret value *" }
                                input {
                                    id: "cs-sec",
                                    r#type: "password",
                                    placeholder: "Paste the secret value here",
                                    value: "{sec_secret}",
                                    oninput: move |e| sec_secret.set(e.value()),
                                }
                            }
                            button {
                                class: "run-btn",
                                disabled: *sec_loading.read(),
                                onclick: run_secret,
                                if *sec_loading.read() { "Storing…" } else { "Store Secret" }
                            }
                            {el}
                        }
                    }
                }
                CredsTab::Cert => {
                    let el = result_element(cert_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            p { class: "section-desc",
                                "Generate an HSM-backed RS256 key pair and self-signed certificate. "
                                "Upload the resulting PEM file to Entra ID under Certificates & secrets."
                            }
                            div { class: "form-group",
                                label { r#for: "cc-cid", "Client ID *" }
                                input {
                                    id: "cc-cid",
                                    r#type: "text",
                                    placeholder: "Application (client) ID",
                                    value: "{cert_client_id}",
                                    oninput: move |e| cert_client_id.set(e.value()),
                                }
                            }
                            div { class: "form-group",
                                label { r#for: "cc-days", "Valid days *" }
                                input {
                                    id: "cc-days",
                                    r#type: "number",
                                    placeholder: "365",
                                    value: "{cert_valid_days}",
                                    oninput: move |e| cert_valid_days.set(e.value()),
                                }
                            }
                            div { class: "form-group",
                                label { r#for: "cc-out", "Output PEM path *" }
                                input {
                                    id: "cc-out",
                                    r#type: "text",
                                    placeholder: "/tmp/cert.pem",
                                    value: "{cert_out}",
                                    oninput: move |e| cert_out.set(e.value()),
                                }
                            }
                            button {
                                class: "run-btn",
                                disabled: *cert_loading.read(),
                                onclick: run_cert,
                                if *cert_loading.read() { "Generating…" } else { "Generate Certificate" }
                            }
                            {el}
                        }
                    }
                }
                CredsTab::Delete => {
                    let el = result_element(del_result.read().clone());
                    let confirming = *del_confirm.read();
                    rsx! {
                        div { class: "form-section",
                            div { class: "confirm-warning",
                                "⚠ This permanently removes credentials from Himmelblau's encrypted cache."
                            }
                            div { class: "checkbox-group",
                                label {
                                    input {
                                        r#type: "checkbox",
                                        checked: *del_secret_only.read(),
                                        onchange: move |e| del_secret_only.set(e.value() == "true"),
                                    }
                                    " Delete secret only"
                                }
                                label {
                                    input {
                                        r#type: "checkbox",
                                        checked: *del_cert_only.read(),
                                        onchange: move |e| del_cert_only.set(e.value() == "true"),
                                    }
                                    " Delete certificate only"
                                }
                            }
                            p { class: "field-hint", "Leave both unchecked to delete all credentials for this domain." }
                            if confirming {
                                div { class: "confirm-row",
                                    button {
                                        class: "run-btn danger-btn",
                                        onclick: run_delete,
                                        "Confirm Delete"
                                    }
                                    button {
                                        class: "cancel-btn",
                                        onclick: move |_| del_confirm.set(false),
                                        "Cancel"
                                    }
                                }
                            } else {
                                button {
                                    class: "run-btn danger-btn",
                                    disabled: *del_loading.read(),
                                    onclick: run_delete,
                                    if *del_loading.read() { "Deleting…" } else { "Delete Credentials" }
                                }
                            }
                            {el}
                        }
                    }
                }
            }}
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
