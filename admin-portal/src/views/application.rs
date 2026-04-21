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
enum AppTab { List, Create, AddSchema, ListSchema }

#[component]
pub fn ApplicationView(username: String) -> Element {
    let mut tab = use_signal(|| AppTab::List);

    // Shared client ID across tabs
    let mut client_id = use_signal(String::new);

    // List tab
    let mut list_name = use_signal(|| username.clone());
    let mut list_loading = use_signal(|| false);
    let mut list_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    // Create tab
    let mut display_name = use_signal(String::new);
    let mut redirect_uris = use_signal(String::new);
    let mut user_rw = use_signal(|| false);
    let mut group_rw = use_signal(|| false);
    let mut create_name = use_signal(|| username.clone());
    let mut create_loading = use_signal(|| false);
    let mut create_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    // Schema tabs (shared)
    let mut schema_obj_id = use_signal(String::new);
    let mut schema_name = use_signal(|| username.clone());
    let mut schema_loading = use_signal(|| false);
    let mut schema_result: Signal<Option<Result<String, String>>> = use_signal(|| None);

    let run_list = move |_| async move {
        let cid = client_id.read().trim().to_string();
        if cid.is_empty() {
            list_result.set(Some(Err("Client ID is required".into())));
            return;
        }
        list_loading.set(true);
        #[derive(Serialize)]
        struct Args { client_id: String, name: Option<String> }
        let args = serde_wasm_bindgen::to_value(&Args {
            client_id: cid,
            name: to_opt(list_name.read().clone()),
        }).unwrap_or(JsValue::NULL);
        match invoke("aad_tool_application_list", args).await {
            Ok(js) => list_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => list_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        list_loading.set(false);
    };

    let run_create = move |_| async move {
        let cid = client_id.read().trim().to_string();
        let dn  = display_name.read().trim().to_string();
        if cid.is_empty() || dn.is_empty() {
            create_result.set(Some(Err("Client ID and Display Name are required".into())));
            return;
        }
        let uris: Vec<String> = redirect_uris.read()
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        create_loading.set(true);
        #[derive(Serialize)]
        struct Args {
            client_id: String,
            display_name: String,
            redirect_uris: Vec<String>,
            user_read_write: bool,
            group_read_write: bool,
            name: Option<String>,
        }
        let args = serde_wasm_bindgen::to_value(&Args {
            client_id: cid,
            display_name: dn,
            redirect_uris: uris,
            user_read_write: *user_rw.read(),
            group_read_write: *group_rw.read(),
            name: to_opt(create_name.read().clone()),
        }).unwrap_or(JsValue::NULL);
        match invoke("aad_tool_application_create", args).await {
            Ok(js) => create_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => create_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        create_loading.set(false);
    };

    let run_schema = move |cmd: &'static str| async move {
        let cid = client_id.read().trim().to_string();
        let oid = schema_obj_id.read().trim().to_string();
        if cid.is_empty() || oid.is_empty() {
            schema_result.set(Some(Err("Client ID and Schema App Object ID are required".into())));
            return;
        }
        schema_loading.set(true);
        #[derive(Serialize)]
        struct Args { client_id: String, schema_app_object_id: String, name: Option<String> }
        let args = serde_wasm_bindgen::to_value(&Args {
            client_id: cid,
            schema_app_object_id: oid,
            name: to_opt(schema_name.read().clone()),
        }).unwrap_or(JsValue::NULL);
        match invoke(cmd, args).await {
            Ok(js) => schema_result.set(Some(Ok(js.as_string().unwrap_or_default()))),
            Err(e) => schema_result.set(Some(Err(e.as_string().unwrap_or_else(|| "Command failed".into())))),
        }
        schema_loading.set(false);
    };

    let run_list_schema  = move |_| run_schema("aad_tool_application_list_schema");
    let run_add_schema   = move |_| run_schema("aad_tool_application_add_schema");

    let active = tab.read().clone();

    rsx! {
        div { class: "view-panel",
            div { class: "view-header",
                h2 { "Applications" }
                p { "Manage Entra ID application registrations and schema extensions." }
            }

            // Shared client ID
            div { class: "form-group shared-field",
                label { r#for: "app-cid", "Client ID (shared across all tabs) *" }
                input {
                    id: "app-cid",
                    r#type: "text",
                    placeholder: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
                    value: "{client_id}",
                    oninput: move |e| client_id.set(e.value()),
                }
            }

            // Tabs
            div { class: "tab-bar",
                button {
                    class: if active == AppTab::List { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(AppTab::List),
                    "List Apps"
                }
                button {
                    class: if active == AppTab::Create { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(AppTab::Create),
                    "Create App"
                }
                button {
                    class: if active == AppTab::ListSchema { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(AppTab::ListSchema),
                    "List Schema"
                }
                button {
                    class: if active == AppTab::AddSchema { "tab-btn tab-active" } else { "tab-btn" },
                    onclick: move |_| tab.set(AppTab::AddSchema),
                    "Add Schema"
                }
            }

            // Tab content
            {match active {
                AppTab::List => {
                    let el = result_element(list_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            div { class: "form-group",
                                label { r#for: "al-name", "Authenticate as (optional)" }
                                input {
                                    id: "al-name",
                                    r#type: "text",
                                    placeholder: "user@domain.com",
                                    value: "{list_name}",
                                    oninput: move |e| list_name.set(e.value()),
                                }
                            }
                            button {
                                class: "run-btn",
                                disabled: *list_loading.read(),
                                onclick: run_list,
                                if *list_loading.read() { "Listing…" } else { "List Applications" }
                            }
                            {el}
                        }
                    }
                }
                AppTab::Create => {
                    let el = result_element(create_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            div { class: "form-group",
                                label { r#for: "ac-dn", "Display name *" }
                                input {
                                    id: "ac-dn",
                                    r#type: "text",
                                    placeholder: "My Admin App",
                                    value: "{display_name}",
                                    oninput: move |e| display_name.set(e.value()),
                                }
                            }
                            div { class: "form-group",
                                label { r#for: "ac-uris", "Redirect URIs (one per line, optional)" }
                                textarea {
                                    id: "ac-uris",
                                    rows: "3",
                                    placeholder: "https://example.com/callback",
                                    value: "{redirect_uris}",
                                    oninput: move |e| redirect_uris.set(e.value()),
                                }
                            }
                            div { class: "checkbox-group",
                                label {
                                    input {
                                        r#type: "checkbox",
                                        checked: *user_rw.read(),
                                        onchange: move |e| user_rw.set(e.value() == "true"),
                                    }
                                    " Grant User.ReadWrite.All"
                                }
                                label {
                                    input {
                                        r#type: "checkbox",
                                        checked: *group_rw.read(),
                                        onchange: move |e| group_rw.set(e.value() == "true"),
                                    }
                                    " Grant Group.ReadWrite.All"
                                }
                            }
                            div { class: "form-group",
                                label { r#for: "ac-name", "Authenticate as (optional)" }
                                input {
                                    id: "ac-name",
                                    r#type: "text",
                                    placeholder: "user@domain.com",
                                    value: "{create_name}",
                                    oninput: move |e| create_name.set(e.value()),
                                }
                            }
                            button {
                                class: "run-btn",
                                disabled: *create_loading.read(),
                                onclick: run_create,
                                if *create_loading.read() { "Creating…" } else { "Create Application" }
                            }
                            {el}
                        }
                    }
                }
                AppTab::ListSchema => {
                    let el = result_element(schema_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            div { class: "form-group",
                                label { r#for: "ls-oid", "Schema App Object ID *" }
                                input {
                                    id: "ls-oid",
                                    r#type: "text",
                                    placeholder: "Object ID of the schema app",
                                    value: "{schema_obj_id}",
                                    oninput: move |e| schema_obj_id.set(e.value()),
                                }
                            }
                            div { class: "form-group",
                                label { r#for: "ls-name", "Authenticate as (optional)" }
                                input {
                                    id: "ls-name",
                                    r#type: "text",
                                    placeholder: "user@domain.com",
                                    value: "{schema_name}",
                                    oninput: move |e| schema_name.set(e.value()),
                                }
                            }
                            button {
                                class: "run-btn",
                                disabled: *schema_loading.read(),
                                onclick: run_list_schema,
                                if *schema_loading.read() { "Listing…" } else { "List Schema Extensions" }
                            }
                            {el}
                        }
                    }
                }
                AppTab::AddSchema => {
                    let el = result_element(schema_result.read().clone());
                    rsx! {
                        div { class: "form-section",
                            p { class: "section-desc",
                                "Registers POSIX extension attributes (uidNumber, gidNumber, etc.) "
                                "on the specified application."
                            }
                            div { class: "form-group",
                                label { r#for: "as-oid", "Schema App Object ID *" }
                                input {
                                    id: "as-oid",
                                    r#type: "text",
                                    placeholder: "Object ID of the schema app",
                                    value: "{schema_obj_id}",
                                    oninput: move |e| schema_obj_id.set(e.value()),
                                }
                            }
                            div { class: "form-group",
                                label { r#for: "as-name", "Authenticate as (optional)" }
                                input {
                                    id: "as-name",
                                    r#type: "text",
                                    placeholder: "user@domain.com",
                                    value: "{schema_name}",
                                    oninput: move |e| schema_name.set(e.value()),
                                }
                            }
                            button {
                                class: "run-btn",
                                disabled: *schema_loading.read(),
                                onclick: run_add_schema,
                                if *schema_loading.read() { "Registering…" } else { "Add Schema Extensions" }
                            }
                            {el}
                        }
                    }
                }
            }}
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
