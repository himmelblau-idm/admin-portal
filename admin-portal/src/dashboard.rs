#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Dashboard(
    name: String,
    username: String,
    access_token: String,
    on_logout: EventHandler,
) -> Element {
    // Show only the first/last 8 chars of the token for display
    let token_preview = if access_token.len() > 20 {
        format!("{}…{}", &access_token[..8], &access_token[access_token.len() - 8..])
    } else {
        access_token.clone()
    };

    rsx! {
        div {
            class: "dashboard",
            header {
                class: "dashboard-header",
                h1 { "Admin Portal" }
                div {
                    class: "dashboard-user",
                    div { class: "auth-badge", "✓ Entra ID" }
                    span { "{name}" }
                    button {
                        class: "logout-btn",
                        onclick: move |_| on_logout.call(()),
                        "Sign Out"
                    }
                }
            }
            main {
                class: "dashboard-main",
                h2 { "Welcome, {name}!" }
                div { class: "auth-info-card",
                    h3 { "Authentication Details" }
                    div { class: "auth-info-row",
                        span { class: "auth-info-label", "Account" }
                        span { class: "auth-info-value", "{username}" }
                    }
                    div { class: "auth-info-row",
                        span { class: "auth-info-label", "Provider" }
                        span { class: "auth-info-value", "Microsoft Entra ID via Himmelblau" }
                    }
                    div { class: "auth-info-row",
                        span { class: "auth-info-label", "Token" }
                        span { class: "auth-info-value token-preview", "{token_preview}" }
                    }
                }
                p { class: "dashboard-placeholder", "Dashboard content will appear here." }
            }
        }
    }
}
