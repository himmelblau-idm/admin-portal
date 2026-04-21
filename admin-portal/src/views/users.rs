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
pub fn UsersView(username: String) -> Element {
    let mut schema_client_id = use_signal(String::new);
    let mut user_id = use_signal(String::new);
    let mut uid = use_signal(String::new);
    let mut gid = use_signal(String::new);
    let mut home = use_signal(String::new);
    let mut shell = use_signal(|| "/bin/bash".to_string());
    let mut gecos = use_signal(String::new);
    let mut name = use_signal(|| username.clone());
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run = move |_| async move {
        let scid = schema_client_id.read().trim().to_string();
        let uid_val = user_id.read().trim().to_string();
        if scid.is_empty() || uid_val.is_empty() {
            result.set(Some(Err("Schema Client ID and User ID are required".into())));
            return;
        }

        let uid_num = parse_opt_u32(&uid.read(), "UID");
        let gid_num = parse_opt_u32(&gid.read(), "GID");

        if let Err(e) = &uid_num { result.set(Some(Err(e.clone()))); return; }
        if let Err(e) = &gid_num { result.set(Some(Err(e.clone()))); return; }

        loading.set(true);
        #[derive(Serialize)]
        struct Args {
            schema_client_id: String,
            user_id: String,
            uid: Option<u32>,
            gid: Option<u32>,
            home: Option<String>,
            shell: Option<String>,
            gecos: Option<String>,
            name: Option<String>,
        }
        let args = serde_wasm_bindgen::to_value(&Args {
            schema_client_id: scid,
            user_id: uid_val,
            uid: uid_num.unwrap(),
            gid: gid_num.unwrap(),
            home:  to_opt(home.read().clone()),
            shell: to_opt(shell.read().clone()),
            gecos: to_opt(gecos.read().clone()),
            name:  to_opt(name.read().clone()),
        }).unwrap_or(JsValue::NULL);

        match invoke("aad_tool_user_set_posix", args).await {
            Ok(js) => result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        loading.set(false);
    };

    let result_el = result_element(result.read().clone());

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "User POSIX Attributes" }
                p {
                    "Set POSIX attributes (UID, GID, home, shell, gecos) on an Entra ID user. "
                    "The schema client application must have User.ReadWrite.All permissions."
                }
            }
            div { class: "form-section",
                div { class: "form-group",
                    label { r#for: "u-scid", "Schema Client ID *" }
                    input {
                        id: "u-scid",
                        r#type: "text",
                        placeholder: "Client ID of the schema registration app",
                        value: "{schema_client_id}",
                        oninput: move |e| schema_client_id.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "u-uid-obj", "User ID (Object ID or UPN) *" }
                    input {
                        id: "u-uid-obj",
                        r#type: "text",
                        placeholder: "user@domain.com or GUID",
                        value: "{user_id}",
                        oninput: move |e| user_id.set(e.value()),
                    }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { r#for: "u-uid", "UID (optional)" }
                        input {
                            id: "u-uid",
                            r#type: "number",
                            placeholder: "e.g. 1001",
                            value: "{uid}",
                            oninput: move |e| uid.set(e.value()),
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "u-gid", "GID (optional)" }
                        input {
                            id: "u-gid",
                            r#type: "number",
                            placeholder: "e.g. 1001",
                            value: "{gid}",
                            oninput: move |e| gid.set(e.value()),
                        }
                    }
                }
                div { class: "form-group",
                    label { r#for: "u-home", "Home directory (optional)" }
                    input {
                        id: "u-home",
                        r#type: "text",
                        placeholder: "/home/username",
                        value: "{home}",
                        oninput: move |e| home.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "u-shell", "Login shell (optional)" }
                    input {
                        id: "u-shell",
                        r#type: "text",
                        placeholder: "/bin/bash",
                        value: "{shell}",
                        oninput: move |e| shell.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "u-gecos", "GECOS / display name (optional)" }
                    input {
                        id: "u-gecos",
                        r#type: "text",
                        placeholder: "Full Name",
                        value: "{gecos}",
                        oninput: move |e| gecos.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { r#for: "u-name", "Authenticate as (leave blank for current user)" }
                    input {
                        id: "u-name",
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

fn parse_opt_u32(s: &str, label: &str) -> Result<Option<u32>, String> {
    let s = s.trim();
    if s.is_empty() { return Ok(None); }
    s.parse::<u32>().map(Some).map_err(|_| format!("Invalid {label}: must be a positive integer"))
}

fn result_element(r: Option<Result<String, String>>) -> Element {
    match r {
        Some(Ok(s))  => rsx! { pre { class: "result-box result-success", "{s}" } },
        Some(Err(e)) => rsx! { pre { class: "result-box result-error",   "{e}" } },
        None         => rsx! {},
    }
}
