# Himmelblau Admin Portal

[![License: GPL v3](https://img.shields.io/badge/License-GPL_v3-blue.svg)](LICENSE)

A desktop GUI for administering [Himmelblau](https://github.com/himmelblau-idm/himmelblau) — the Microsoft Entra ID (Azure AD) identity broker for Linux. All the power of `aad-tool` in a point-and-click interface, with privileged operations handled transparently via polkit.

## Why use it?

- **No terminal required** — manage Entra ID users, groups, app registrations, credentials, and PAM configuration from a clean dashboard.
- **Safe privilege escalation** — operations that require root invoke `pkexec` automatically; you never have to run the app itself as root.
- **Seamless authentication** — signs in through the Himmelblau broker over D-Bus; silent token refresh happens in the background so you are never asked to log in twice.
- **Instant feedback** — every command streams stdout/stderr into the UI in real time, so you always know what happened.
- **Offline safety** — dedicated breakglass controls let you activate cached-credential mode during Entra ID outages without touching a config file.

## Prerequisites

| Requirement | Version |
|---|---|
| [Rust toolchain](https://rustup.rs) | stable ≥ 1.85 (edition 2024) |
| [Dioxus CLI](https://dioxuslabs.com/learn/0.6/CLI/installation) (`dx`) | 0.7 |
| [Tauri CLI](https://tauri.app/start/prerequisites/) (`cargo tauri`) | 2 |
| [Himmelblau](https://github.com/himmelblau-idm/himmelblau) & `aad-tool` | installed on the host |
| polkit / `pkexec` | any version |

Install the CLIs:

```bash
cargo install dioxus-cli
cargo install tauri-cli --version "^2"
```

## Quick start

```bash
# Clone and enter the project
git clone <repo-url>
cd admin-portal

# Launch in development mode (hot-reload)
cargo tauri dev
```

A native window opens. Sign in with your Microsoft Entra ID account — a PIN dialog may appear if Windows Hello is configured.

> **Do not run with `sudo`.** Himmelblau requires a D-Bus session bus that is stripped when running as root. Operations that need elevated privileges will prompt for authentication automatically.

## Building for release

```bash
cargo tauri build
```

The distributable bundle (`.deb`, `.rpm`, AppImage, etc.) is written to `src-tauri/target/release/bundle/`.

---

## Dashboard sections

### System

#### Status & System
Check the live status of the `himmelblaud` daemon, query TPM key-storage availability, and display the installed `aad-tool` version — all without leaving the app.

### Management

#### Applications
Manage Entra ID application registrations from four tabs:

| Tab | What it does |
|---|---|
| **List Apps** | List all app registrations visible to the authenticated account |
| **Create App** | Register a new application, set redirect URIs, and optionally grant `User.ReadWrite.All` or `Group.ReadWrite.All` |
| **List Schema** | Inspect schema extensions on a schema app |
| **Add Schema** | Register POSIX extension attributes (`uidNumber`, `gidNumber`, etc.) on an application |

A shared **Client ID** field at the top applies to every tab so you only type it once.

#### Users — POSIX Attributes
Set Linux-compatible attributes on an Entra ID user account:

- UID and primary GID
- Home directory
- Login shell (defaults to `/bin/bash`)
- GECOS / display name

Requires a schema client application with `User.ReadWrite.All` permissions.

#### Groups — POSIX Attributes
Assign a `gidNumber` to an Entra ID group (identified by Object ID). Requires a schema client application with `Group.ReadWrite.All` permissions.

### Identity

#### ID Mapping
Maintain a static UID/GID lookup table — useful when migrating from on-premises Active Directory and you need deterministic numeric IDs:

- **Add User** — map a UPN or SAM-compatible name to a fixed UID and primary GID.
- **Add Group** — map a group Object ID to a fixed GID.
- **Clear Cache** — wipe all static mappings (requires confirmation).

All ID map operations run via `pkexec` and require a polkit authentication dialog.

### Security

#### Credentials
Manage the confidential client credentials that `himmelblaud` uses to authenticate to Entra ID:

| Tab | What it does |
|---|---|
| **List** | Check whether a client secret and/or certificate exists for a domain |
| **Add Secret** | Store a client secret copied from the Entra ID portal |
| **Add Certificate** | Generate an HSM-backed RS256 key pair and self-signed certificate, then export a PEM file to upload to Entra ID |
| **Delete** | Remove stored secrets, certificates, or both (requires confirmation) |

A shared **Domain** field applies to all tabs. All write operations run via `pkexec`.

#### PAM Configuration
Configure `pam_himmelblau` in your system PAM files. Optionally supply custom paths for the auth, account, session, and password stacks. Without the **Apply changes** checkbox the command runs in dry-run mode — safe to preview before committing.

Runs via `pkexec`.

#### Offline Breakglass
Activate Himmelblau's offline breakglass mode so cached Entra ID credentials continue to work when Azure is unreachable. Accepts a TTL with time suffixes (`m`, `h`, `d`); pass `0` to immediately exit breakglass mode.

Must be pre-enabled in `himmelblau.conf`. Runs via `pkexec`.

### Diagnostics

#### Cache
Two sub-operations:

- **Cache Clear** — mark cached user/group entries as stale. Three granularity flags:
  - *Only clear NSS cache*
  - *Only clear mapped-name cache*
  - *Full wipe* — also unjoins the host from Entra ID (irreversible; requires an extra confirmation step)
- **Enumerate** — pull all users and groups with rfc2307 attributes from Entra ID and cache them locally. Accepts an optional Client ID and account name filter.

Runs via `pkexec`.

#### Auth Test
Test that `himmelblaud` correctly processes a PAM authentication for a given account and password/PIN. The credential is piped directly to the daemon over a private channel — it is never stored or logged.

---

## Architecture

```
┌──────────────────────────────────────────┐
│            Tauri native window           │
│  ┌────────────────────────────────────┐  │
│  │   Dioxus UI (WebAssembly / web)    │  │
│  │   src/{app,login,dashboard,views}  │  │
│  └────────────────┬───────────────────┘  │
│                   │  window.__TAURI__     │
│  ┌────────────────▼───────────────────┐  │
│  │       Tauri backend (Rust)         │  │
│  │   src-tauri/src/lib.rs             │  │
│  │   ├─ broker.rs  (D-Bus / zbus)     │  │
│  │   └─ aad_tool.rs (subprocess)      │  │
│  └───────────┬──────────────┬─────────┘  │
└──────────────┼──────────────┼────────────┘
               │              │
     D-Bus session        pkexec / aad-tool
               │              │
   ┌───────────▼──┐   ┌───────▼──────────┐
   │ himmelblaud  │   │  aad-tool (root) │
   │  (broker)    │   │                  │
   └──────────────┘   └──────────────────┘
```

| Layer | Tech | Role |
|---|---|---|
| UI | Dioxus 0.7 → WASM | Reactive component tree, form state, async Tauri calls |
| Shell | Tauri 2 | Native window, IPC bridge, command routing |
| Auth | zbus 5 + D-Bus | Talks to `com.microsoft.identity.broker1` for token acquisition |
| CLI bridge | Tokio + subprocess | Wraps `aad-tool`; escalates via `pkexec` for privileged commands |

### Authentication flow

1. On startup the app calls `get_accounts` on the broker.
2. If a cached account exists, `acquire_token_silent` is attempted.
3. On success the Dashboard opens immediately (no sign-in prompt).
4. If silent auth fails, the Login page becomes interactive and the user clicks **Sign in with Microsoft** → `acquire_token_interactive` → broker opens a PIN/passphrase dialog.

### Privilege model

Commands are split into two tiers:

- **Unprivileged** (`run_aad_tool`) — reads only; runs as the current user.
- **Privileged** (`run_aad_tool_as_root`) — writes or reads sensitive data; runs via `pkexec /usr/bin/aad-tool …`. A polkit agent dialog is shown to the user by the desktop environment.

The UI marks privileged sections with a `◆` badge and shows an admin notice banner inside the content area.

---

## Development

### IDE setup

[VS Code](https://code.visualstudio.com/) with:
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Dioxus](https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus)

### Project layout

```
admin-portal/
├── src/                  # Dioxus frontend (compiled to WASM)
│   ├── main.rs           # Entry point — initialises logger, launches App
│   ├── app.rs            # Root component: auth state machine, route switching
│   ├── login.rs          # Login page component
│   ├── dashboard.rs      # Sidebar navigation + content router
│   └── views/            # One file per dashboard section
│       ├── status.rs
│       ├── cache.rs
│       ├── application.rs
│       ├── users.rs
│       ├── groups.rs
│       ├── idmap.rs
│       ├── creds.rs
│       ├── pam.rs
│       ├── auth_test.rs
│       └── breakglass.rs
├── src-tauri/            # Tauri backend (native Rust)
│   └── src/
│       ├── lib.rs        # Tauri commands wired to invoke_handler
│       ├── broker.rs     # D-Bus broker client (zbus)
│       └── aad_tool.rs   # aad-tool subprocess helpers
├── assets/
│   └── styles.css        # All UI styles
├── Cargo.toml            # Frontend crate (admin-portal-ui)
└── Dioxus.toml           # Dioxus CLI configuration
```

### Adding a new command

1. **Backend** — add a `#[tauri::command]` function in `src-tauri/src/lib.rs` and register it in `tauri::generate_handler![…]`.
2. **Frontend** — call `invoke("your_command_name", args)` from a Dioxus view component, following the pattern in any existing view file.

### Hot-reload

`cargo tauri dev` starts both the Dioxus dev server (`dx serve --port 1420`) and the Tauri process. Editing any `.rs` file in `src/` triggers an automatic WASM rebuild and page reload. Changes to `src-tauri/src/` restart the Tauri backend.

---

## License

This project is licensed under the **GNU General Public License v3.0**. See [LICENSE](LICENSE) for the full text.

    Himmelblau Admin Portal — a desktop GUI for the Himmelblau identity broker.
    Copyright (C) 2026  the Himmelblau Admin Portal contributors

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

