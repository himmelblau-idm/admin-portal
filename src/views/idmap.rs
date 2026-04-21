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
enum IdMapTab { UserAdd, GroupAdd, Clear }

#[component]
pub fn IdMapView() -> Element {
    let mut tab = use_signal(|| IdMapTab::UserAdd);

    // User add
    let mut ua_name = use_signal(String::new);
    let mut ua_uid  = use_signal(String::new);
    let mut ua_gid  = use_signal(String::new);
    let mut ua_loading = use_signal(|| false);
    let mut ua_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    // Group add
    let mut ga_oid = use_signal(String::new);
    let mut ga_gid = use_signal(String::new);
    let mut ga_loading = use_signal(|| false);
    let mut ga_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    // Clear
    let mut cl_confirm = use_signal(|| false);
    let mut cl_loading = use_signal(|| false);
    let mut cl_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run_user_add = move |_| async move {
        let nm  = ua_name.read().trim().to_string();
        let uid_str = ua_uid.read().trim().to_string();
        let gid_str = ua_gid.read().trim().to_string();
        if nm.is_empty() || uid_str.is_empty() || gid_str.is_empty() {
            ua_result.set(Some(Err("Name, UID, and GID are all required".into())));
            return;
        }
        let uid: u32 = match uid_str.parse() { Ok(v) => v, Err(_) => { ua_result.set(Some(Err("UID must be a positive integer".into()))); return; } };
        let gid: u32 = match gid_str.parse() { Ok(v) => v, Err(_) => { ua_result.set(Some(Err("GID must be a positive integer".into()))); return; } };
        ua_loading.set(true);
        #[derive(Serialize)]
        struct Args { account_name: String, uid: u32, gid: u32 }
        let args = serde_wasm_bindgen::to_value(&Args { account_name: nm, uid, gid })
            .unwrap_or(JsValue::NULL);
        match invoke("aad_tool_idmap_user_add", args).await {
            Ok(js) => ua_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => ua_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        ua_loading.set(false);
    };

    let run_group_add = move |_| async move {
        let oid = ga_oid.read().trim().to_string();
        let gid_str = ga_gid.read().trim().to_string();
        if oid.is_empty() || gid_str.is_empty() {
            ga_result.set(Some(Err("Object ID and GID are required".into())));
            return;
        }
        let gid: u32 = match gid_str.parse() { Ok(v) => v, Err(_) => { ga_result.set(Some(Err("GID must be a positive integer".into()))); return; } };
        ga_loading.set(true);
        #[derive(Serialize)]
        struct Args { object_id: String, gid: u32 }
        let args = serde_wasm_bindgen::to_value(&Args { object_id: oid, gid })
            .unwrap_or(JsValue::NULL);
        match invoke("aad_tool_idmap_group_add", args).await {
            Ok(js) => ga_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => ga_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        ga_loading.set(false);
    };

    let run_clear = move |_| async move {
        if !*cl_confirm.read() {
            cl_confirm.set(true);
            return;
        }
        cl_confirm.set(false);
        cl_loading.set(true);
        let empty = js_sys::Object::new();
        match invoke("aad_tool_idmap_clear", empty.into()).await {
            Ok(js) => cl_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => cl_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        cl_loading.set(false);
    };

    let active = tab.read().clone();

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "ID Mapping" }
                p {
                    "Manage the static idmap cache — map Entra ID accounts to fixed UID/GID values. "
                    "Useful when migrating from on-prem AD. Requires elevated privileges (pkexec)."
                }
            }
            div { class: "tab-bar",
                button {
                    class: if active == IdMapTab::UserAdd { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(IdMapTab::UserAdd),
                    "Add User"
                }
                button {
                    class: if active == IdMapTab::GroupAdd { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(IdMapTab::GroupAdd),
                    "Add Group"
                }
                button {
                    class: if active == IdMapTab::Clear { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(IdMapTab::Clear),
                    "Clear Cache"
                }
            }
            {match active {
                IdMapTab::UserAdd => {
                    let el = result_element(ua_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            p { class: "section-desc", "Map an Entra ID user (UPN or SAM-compatible name) to a static UID and primary GID." }
                            div { class: "form-group",
                                label { r#for: "ua-name", "Account (UPN or name) *" }
                                input {
                                    id: "ua-name",
                                    r#type: "text",
                                    placeholder: "user@domain.com",
                                    value: "{ua_name}",
                                    oninput: move |e| ua_name.set(e.value()),
                                }
                            }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { r#for: "ua-uid", "UID *" }
                                    input {
                                        id: "ua-uid",
                                        r#type: "number",
                                        placeholder: "1001",
                                        value: "{ua_uid}",
                                        oninput: move |e| ua_uid.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { r#for: "ua-gid", "GID *" }
                                    input {
                                        id: "ua-gid",
                                        r#type: "number",
                                        placeholder: "1001",
                                        value: "{ua_gid}",
                                        oninput: move |e| ua_gid.set(e.value()),
                                    }
                                }
                            }
                            button {
                                class: "run-btn",
                                disabled: *ua_loading.read(),
                                onclick: run_user_add,
                                if *ua_loading.read() { "Adding…" } else { "Add User Mapping" }
                            }
                            {el}
                        }
                    }
                }
                IdMapTab::GroupAdd => {
                    let el = result_element(ga_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            p { class: "section-desc", "Map an Entra ID group (by Object ID) to a static GID." }
                            div { class: "form-group",
                                label { r#for: "ga-oid", "Group Object ID *" }
                                input {
                                    id: "ga-oid",
                                    r#type: "text",
                                    placeholder: "GUID",
                                    value: "{ga_oid}",
                                    oninput: move |e| ga_oid.set(e.value()),
                                }
                            }
                            div { class: "form-group",
                                label { r#for: "ga-gid", "GID *" }
                                input {
                                    id: "ga-gid",
                                    r#type: "number",
                                    placeholder: "1001",
                                    value: "{ga_gid}",
                                    oninput: move |e| ga_gid.set(e.value()),
                                }
                            }
                            button {
                                class: "run-btn",
                                disabled: *ga_loading.read(),
                                onclick: run_group_add,
                                if *ga_loading.read() { "Adding…" } else { "Add Group Mapping" }
                            }
                            {el}
                        }
                    }
                }
                IdMapTab::Clear => {
                    let el = result_element(cl_result.read().clone());
                    let confirming = *cl_confirm.read();
                    rsx! {
                        div { class: "form-section",
                            div { class: "confirm-warning",
                                "⚠ This will clear all static UID/GID mappings in the idmap cache."
                            }
                            if confirming {
                                div { class: "confirm-row",
                                    button {
                                        class: "run-btn danger-btn",
                                        onclick: run_clear,
                                        "Confirm Clear"
                                    }
                                    button {
                                        class: "cancel-btn",
                                        onclick: move |_| cl_confirm.set(false),
                                        "Cancel"
                                    }
                                }
                            } else {
                                button {
                                    class: "run-btn danger-btn",
                                    disabled: *cl_loading.read(),
                                    onclick: run_clear,
                                    if *cl_loading.read() { "Clearing…" } else { "Clear ID Map Cache" }
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
