#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::views::application::ApplicationView;
use crate::views::auth_test::AuthTestView;
use crate::views::breakglass::BreakglassView;
use crate::views::cache::CacheView;
use crate::views::creds::CredsView;
use crate::views::groups::GroupsView;
use crate::views::idmap::IdMapView;
use crate::views::pam::PamView;
use crate::views::status::StatusView;
use crate::views::users::UsersView;

#[derive(Clone, PartialEq)]
enum Section {
    Status,
    Cache,
    Applications,
    Users,
    Groups,
    IdMap,
    Credentials,
    PamConfig,
    AuthTest,
    Breakglass,
}

impl Section {
    fn label(&self) -> &str {
        match self {
            Self::Status       => "Status & System",
            Self::Cache        => "Cache",
            Self::Applications => "Applications",
            Self::Users        => "Users",
            Self::Groups       => "Groups",
            Self::IdMap        => "ID Mapping",
            Self::Credentials  => "Credentials",
            Self::PamConfig    => "PAM Config",
            Self::AuthTest     => "Auth Test",
            Self::Breakglass   => "Offline Breakglass",
        }
    }

    fn icon(&self) -> &str {
        match self {
            Self::Status       => "◉",
            Self::Cache        => "⟳",
            Self::Applications => "⊞",
            Self::Users        => "◎",
            Self::Groups       => "◈",
            Self::IdMap        => "⇄",
            Self::Credentials  => "◆",
            Self::PamConfig    => "≡",
            Self::AuthTest     => "⊙",
            Self::Breakglass   => "⚑",
        }
    }

    fn requires_admin(&self) -> bool {
        matches!(
            self,
            Self::Cache | Self::IdMap | Self::Credentials | Self::PamConfig | Self::Breakglass
        )
    }
}

#[component]
pub fn Dashboard(
    name: String,
    username: String,
    access_token: String,
    on_logout: EventHandler,
) -> Element {
    let mut section = use_signal(|| Section::Status);

    let nav_item = |s: Section, current: &Section| {
        let label = s.label();
        let icon  = s.icon();
        let admin = s.requires_admin();
        let is_active = &s == current;
        let class = if is_active { "nav-item nav-active" } else { "nav-item" };
        rsx! {
            button {
                class,
                onclick: move |_| section.set(s.clone()),
                span { class: "nav-icon", "{icon}" }
                span { class: "nav-label", "{label}" }
                if admin {
                    span { class: "nav-admin-badge", title: "Requires administrator", "◆" }
                }
            }
        }
    };

    let current = section.read().clone();
    let uname = username.clone();

    rsx! {
        div { class: "dashboard",
            // ── Sidebar ────────────────────────────────────────────────────
            nav { class: "sidebar",
                div { class: "sidebar-brand",
                    span { class: "brand-icon", "◈" }
                    span { class: "brand-title", "Admin Portal" }
                }

                div { class: "nav-group",
                    span { class: "nav-group-label", "SYSTEM" }
                    {nav_item(Section::Status, &current)}
                }
                div { class: "nav-group",
                    span { class: "nav-group-label", "MANAGEMENT" }
                    {nav_item(Section::Applications, &current)}
                    {nav_item(Section::Users, &current)}
                    {nav_item(Section::Groups, &current)}
                }
                div { class: "nav-group",
                    span { class: "nav-group-label", "IDENTITY" }
                    {nav_item(Section::IdMap, &current)}
                }
                div { class: "nav-group",
                    span { class: "nav-group-label", "SECURITY" }
                    {nav_item(Section::Credentials, &current)}
                    {nav_item(Section::PamConfig, &current)}
                    {nav_item(Section::Breakglass, &current)}
                }
                div { class: "nav-group",
                    span { class: "nav-group-label", "DIAGNOSTICS" }
                    {nav_item(Section::Cache, &current)}
                    {nav_item(Section::AuthTest, &current)}
                }

                div { class: "sidebar-footer",
                    div { class: "sidebar-user",
                        div { class: "auth-badge", "● Entra ID" }
                        span { class: "sidebar-username", "{name}" }
                    }
                    button {
                        class: "logout-btn",
                        onclick: move |_| on_logout.call(()),
                        "Sign Out"
                    }
                }
            }

            // ── Content area ───────────────────────────────────────────────
            main { class: "content-area",
                if current.requires_admin() {
                    div { class: "admin-notice",
                        span { class: "admin-notice-icon", "◆" }
                        span {
                            "Operations in this section require administrator privileges. "
                            "A system authentication dialog will appear when you execute an action."
                        }
                    }
                }
                {match current {
                    Section::Status       => rsx! { StatusView {} },
                    Section::Cache        => rsx! { CacheView { username: uname.clone() } },
                    Section::Applications => rsx! { ApplicationView { username: uname.clone() } },
                    Section::Users        => rsx! { UsersView { username: uname.clone() } },
                    Section::Groups       => rsx! { GroupsView { username: uname.clone() } },
                    Section::IdMap        => rsx! { IdMapView {} },
                    Section::Credentials  => rsx! { CredsView {} },
                    Section::PamConfig    => rsx! { PamView {} },
                    Section::AuthTest     => rsx! { AuthTestView { username: uname.clone() } },
                    Section::Breakglass   => rsx! { BreakglassView {} },
                }}
            }
        }
    }
}

