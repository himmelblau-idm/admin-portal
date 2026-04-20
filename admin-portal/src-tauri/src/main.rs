// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Hint GDK to use Adwaita cursors (complete set, even dimensions at all sizes).
    // XCURSOR_SIZE=32 → at Wayland scale 2, GDK requests 64-px cursors; Adwaita
    // ships 64×64 for every cursor, so the "not an integer multiple of scale"
    // warning disappears.
    std::env::set_var("XCURSOR_THEME", "Adwaita");
    std::env::set_var("XCURSOR_SIZE", "32");

    // Safety-net: on GNOME/Wayland the compositor can override XCURSOR_THEME via
    // GSettings.  Install a GLib log handler that silently drops GDK messages about
    // cursors while forwarding every other GDK warning/message to stderr.
    #[cfg(target_os = "linux")]
    suppress_gdk_cursor_noise();

    admin_portal_lib::run()
}

/// Install a GLib log handler that drops GDK cursor-related log lines.
///
/// `libglib-2.0` is already linked transitively via WebKitGTK, so no
/// additional Cargo dependency is needed.
#[cfg(target_os = "linux")]
fn suppress_gdk_cursor_noise() {
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_uint, c_void};

    // GLib log-level flags
    const G_LOG_LEVEL_WARNING: c_uint = 1 << 4;
    const G_LOG_LEVEL_MESSAGE: c_uint = 1 << 5;

    extern "C" {
        fn g_log_set_handler(
            log_domain: *const c_char,
            log_levels: c_uint,
            log_func: unsafe extern "C" fn(*const c_char, c_uint, *const c_char, *mut c_void),
            user_data: *mut c_void,
        ) -> c_uint;
    }

    unsafe extern "C" fn handler(
        _domain: *const c_char,
        level: c_uint,
        message: *const c_char,
        _data: *mut c_void,
    ) {
        if message.is_null() {
            return;
        }
        let msg = unsafe { CStr::from_ptr(message) }.to_string_lossy();
        // Drop cursor-related noise; forward everything else to stderr.
        if msg.contains("cursor") || msg.contains("Cursor") {
            return;
        }
        let level_str = if level & (1 << 4) != 0 { "WARNING" } else { "Message" };
        eprintln!("Gdk-{level_str}: {msg}");
    }

    let domain = CString::new("Gdk").expect("static string");
    unsafe {
        g_log_set_handler(
            domain.as_ptr(),
            G_LOG_LEVEL_WARNING | G_LOG_LEVEL_MESSAGE,
            handler,
            std::ptr::null_mut(),
        );
    }
}
