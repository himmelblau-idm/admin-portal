#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn StatusView() -> Element {
    let mut status_result: Signal<Option<Result<String, String>>> = use_signal(|| None);
    let mut tpm_result: Signal<Option<Result<String, String>>> = use_signal(|| None);
    let mut version_result: Signal<Option<Result<String, String>>> = use_signal(|| None);
    let mut status_loading = use_signal(|| false);
    let mut tpm_loading = use_signal(|| false);
    let mut version_loading = use_signal(|| false);

    let run_status = move |_| async move {
        status_loading.set(true);
        let empty = js_sys::Object::new();
        match invoke("aad_tool_status", empty.into()).await {
            Ok(js) => status_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => status_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        status_loading.set(false);
    };

    let run_tpm = move |_| async move {
        tpm_loading.set(true);
        let empty = js_sys::Object::new();
        match invoke("aad_tool_tpm", empty.into()).await {
            Ok(js) => tpm_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => tpm_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        tpm_loading.set(false);
    };

    let run_version = move |_| async move {
        version_loading.set(true);
        let empty = js_sys::Object::new();
        match invoke("aad_tool_version", empty.into()).await {
            Ok(js) => version_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => version_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        version_loading.set(false);
    };

    let status_el = result_element(status_result.read().clone());
    let tpm_el = result_element(tpm_result.read().clone());
    let version_el = result_element(version_result.read().clone());

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "Status & System" }
                p { "Check the himmelblaud daemon, TPM status, and installed tool version." }
            }
            div { class: "status-cards",
                div { class: "status-card",
                    h3 { "Daemon Status" }
                    p { class: "status-desc",
                        "Verifies that himmelblaud is online and can connect to Entra ID."
                    }
                    button {
                        class: "run-btn",
                        disabled: *status_loading.read(),
                        onclick: run_status,
                        if *status_loading.read() { "Checking…" } else { "Check Status" }
                    }
                    {status_el}
                }
                div { class: "status-card",
                    h3 { "TPM Status" }
                    p { class: "status-desc",
                        "Checks whether Himmelblau is utilizing the TPM for key storage."
                    }
                    button {
                        class: "run-btn",
                        disabled: *tpm_loading.read(),
                        onclick: run_tpm,
                        if *tpm_loading.read() { "Checking…" } else { "Check TPM" }
                    }
                    {tpm_el}
                }
                div { class: "status-card",
                    h3 { "Tool Version" }
                    p { class: "status-desc", "Shows the installed version of aad-tool." }
                    button {
                        class: "run-btn",
                        disabled: *version_loading.read(),
                        onclick: run_version,
                        if *version_loading.read() { "Checking…" } else { "Show Version" }
                    }
                    {version_el}
                }
            }
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
