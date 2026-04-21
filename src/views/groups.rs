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
pub fn GroupsView(username: String) -> Element {
    let mut schema_client_id = use_signal(String::new);
    let mut group_id = use_signal(String::new);
    let mut gid = use_signal(String::new);
    let mut name = use_signal(|| username.clone());
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run = move |_| async move {
        let scid = schema_client_id.read().trim().to_string();
        let gid_obj = group_id.read().trim().to_string();
        let gid_str = gid.read().trim().to_string();
        if scid.is_empty() || gid_obj.is_empty() || gid_str.is_empty() {
            result.set(Some(Err("Schema Client ID, Group ID, and GID are all required".into())));
            return;
        }
        let gid_val: u32 = match gid_str.parse() {
            Ok(v) => v,
            Err(_) => {
                result.set(Some(Err("GID must be a positive integer".into())));
                return;
            }
        };
        loading.set(true);
        #[derive(Serialize)]
        struct Args {
            schema_client_id: String,
            group_id: String,
            gid: u32,
            name: Option<String>,
        }
        let args = serde_wasm_bindgen::to_value(&Args {
            schema_client_id: scid,
            group_id: gid_obj,
            gid: gid_val,
            name: to_opt(name.read().clone()),
        }).unwrap_or(JsValue::NULL);
        match invoke("aad_tool_group_set_posix", args).await {
            Ok(js) => result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        loading.set(false);
    };

    let result_el = result_element(result.read().clone());

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "Group POSIX Attributes" }
                p {
                    "Set the gidNumber attribute on an Entra ID group. "
                    "The schema client application must have Group.ReadWrite.All permissions."
                }
            }
            div { class: "form-section",
                div { class: "form-group",
                    label { r#for: "g-scid", "Schema Client ID *" }
                    input {
                        id: "g-scid",
                        r#type: "text",
                        placeholder: "Client ID of the schema registration app",
                        value: "{schema_client_id}",
                        oninput: move |e| schema_client_id.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "g-gid-obj", "Group Object ID *" }
                    input {
                        id: "g-gid-obj",
                        r#type: "text",
                        placeholder: "GUID",
                        value: "{group_id}",
                        oninput: move |e| group_id.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "g-gid", "GID *" }
                    input {
                        id: "g-gid",
                        r#type: "number",
                        placeholder: "e.g. 1001",
                        value: "{gid}",
                        oninput: move |e| gid.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "g-name", "Authenticate as (leave blank for current user)" }
                    input {
                        id: "g-name",
                        r#type: "text",
                        placeholder: "user@domain.com",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }
                button {
                    class: "run-btn",
                    disabled: *loading.read(),
                    onclick: run,
                    if *loading.read() { "Setting attributes…" } else { "Set POSIX Attributes" }
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
