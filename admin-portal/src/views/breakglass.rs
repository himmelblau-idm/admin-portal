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
pub fn BreakglassView() -> Element {
    let mut ttl = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run = move |_| async move {
        loading.set(true);
        #[derive(Serialize)]
        struct Args { ttl: Option<String> }
        let t = ttl.read().trim().to_string();
        let args = serde_wasm_bindgen::to_value(&Args { ttl: to_opt(t) })
            .unwrap_or(JsValue::NULL);
        match invoke("aad_tool_offline_breakglass", args).await {
            Ok(js) => result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        loading.set(false);
    };

    let result_el = result_element(result.read().clone());

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "Offline Breakglass" }
                p {
                    "Activate or deactivate Himmelblau's offline breakglass mode. "
                    "When active, cached Entra ID credentials can be used when Azure is unreachable. "
                    "Breakglass must be pre-enabled in himmelblau.conf. "
                    "Requires elevated privileges (pkexec)."
                }
            }
            div { class: "form-section",
                div { class: "confirm-warning",
                    "⚠ Only use breakglass mode during verified Entra ID outages."
                }
                div { class: "form-group",
                    label { r#for: "bg-ttl", "TTL (leave blank for configured default)" }
                    input {
                        id: "bg-ttl",
                        r#type: "text",
                        placeholder: "e.g. 2h, 1d, or 0 to disable",
                        value: "{ttl}",
                        oninput: move |e| ttl.set(e.value()),
                    }
                    p { class: "field-hint",
                        "Accepts time suffixes: m (minutes), h (hours), d (days). "
                        "Use \"0\" to immediately exit breakglass mode."
                    }
                }
                button {
                    class: "run-btn",
                    disabled: *loading.read(),
                    onclick: run,
                    if *loading.read() { "Activating…" } else { "Activate Breakglass" }
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
