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

/// Suppress GDK cursor-related log noise on Linux.
///
/// Modern GDK (GTK 4 / GLib 2.56+) emits cursor warnings via *structured*
/// logging (`g_log_structured`), which routes through `g_log_set_writer_func`
/// rather than the older `g_log_set_handler`.  We install both:
///
/// * `g_log_set_handler("Gdk", …)` — catches the old-style path.
/// * `g_log_set_writer_func(…)`   — catches the structured path (global sink).
///
/// Both filters look for "cursor" / "Cursor" / "Unable to load" in the message
/// and silently drop matching lines.  Everything else is forwarded to GLib's
/// built-in default writer so application and WebKit logs are unaffected.
///
/// `libglib-2.0` is already linked transitively via WebKitGTK; no extra
/// Cargo dependency is needed.
#[cfg(target_os = "linux")]
fn suppress_gdk_cursor_noise() {
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_uint, c_void};

    // ── GLib log-level flags ────────────────────────────────────────────────
    const G_LOG_FLAG_RECURSION: c_uint = 1 << 0;
    const G_LOG_FLAG_FATAL: c_uint = 1 << 1;
    const G_LOG_LEVEL_ERROR: c_uint = 1 << 2;
    const G_LOG_LEVEL_CRITICAL: c_uint = 1 << 3;
    const G_LOG_LEVEL_WARNING: c_uint = 1 << 4;
    const G_LOG_LEVEL_MESSAGE: c_uint = 1 << 5;
    const G_LOG_LEVEL_INFO: c_uint = 1 << 6;
    const G_LOG_LEVEL_DEBUG: c_uint = 1 << 7;
    const G_LOG_WRITER_HANDLED: c_uint = 1;

    // ── GLogField (structured-logging field: key + value + length) ──────────
    #[repr(C)]
    struct GLogField {
        key: *const c_char,
        value: *const c_void,
        /// Byte length of `value`, or -1 when `value` is a NUL-terminated string.
        length: isize,
    }

    extern "C" {
        // Old-style per-domain handler
        fn g_log_set_handler(
            log_domain: *const c_char,
            log_levels: c_uint,
            log_func: unsafe extern "C" fn(*const c_char, c_uint, *const c_char, *mut c_void),
            user_data: *mut c_void,
        ) -> c_uint;

        // Global structured-log writer (GLib ≥ 2.50)
        fn g_log_set_writer_func(
            func: unsafe extern "C" fn(c_uint, *const GLogField, usize, *mut c_void) -> c_uint,
            user_data: *mut c_void,
            user_data_free: Option<unsafe extern "C" fn(*mut c_void)>,
        );

        // Default writer we delegate non-cursor messages to
        fn g_log_writer_default(
            log_level: c_uint,
            fields: *const GLogField,
            n_fields: usize,
            user_data: *mut c_void,
        ) -> c_uint;
    }

    // ── Old-style handler ───────────────────────────────────────────────────
    // Registered only for the "Gdk" domain; drops cursor messages silently.
    unsafe extern "C" fn old_handler(
        _domain: *const c_char,
        level: c_uint,
        message: *const c_char,
        _data: *mut c_void,
    ) {
        if message.is_null() {
            return;
        }
        let msg = unsafe { CStr::from_ptr(message) }.to_string_lossy();
        if is_cursor_noise(&msg) {
            return;
        }
        let level_str = match level {
            l if l & G_LOG_LEVEL_WARNING != 0 => "WARNING",
            l if l & G_LOG_LEVEL_MESSAGE != 0 => "Message",
            _ => "Info",
        };
        eprintln!("Gdk-{level_str}: {msg}");
    }

    // ── Structured-log writer ───────────────────────────────────────────────
    // Global sink — receives every g_log_structured() call from all domains.
    // Inspect the MESSAGE field; drop cursor noise, forward everything else.
    unsafe extern "C" fn writer_func(
        log_level: c_uint,
        fields: *const GLogField,
        n_fields: usize,
        user_data: *mut c_void,
    ) -> c_uint {
        let fields_slice = unsafe { std::slice::from_raw_parts(fields, n_fields) };
        for field in fields_slice {
            if field.key.is_null() || field.value.is_null() {
                continue;
            }
            let key = unsafe { CStr::from_ptr(field.key) }.to_string_lossy();
            if key == "MESSAGE" {
                let msg: std::borrow::Cow<str> = if field.length < 0 {
                    unsafe { CStr::from_ptr(field.value as *const c_char) }.to_string_lossy()
                } else {
                    let bytes = unsafe {
                        std::slice::from_raw_parts(field.value as *const u8, field.length as usize)
                    };
                    String::from_utf8_lossy(bytes)
                };
                if is_cursor_noise(&msg) {
                    return G_LOG_WRITER_HANDLED;
                }
                break;
            }
        }
        unsafe { g_log_writer_default(log_level, fields, n_fields, user_data) }
    }

    // ── Install both handlers ────────────────────────────────────────────────
    let domain = CString::new("Gdk").expect("static string");
    let all_levels = G_LOG_FLAG_RECURSION
        | G_LOG_FLAG_FATAL
        | G_LOG_LEVEL_ERROR
        | G_LOG_LEVEL_CRITICAL
        | G_LOG_LEVEL_WARNING
        | G_LOG_LEVEL_MESSAGE
        | G_LOG_LEVEL_INFO
        | G_LOG_LEVEL_DEBUG;
    unsafe {
        g_log_set_handler(domain.as_ptr(), all_levels, old_handler, std::ptr::null_mut());
        g_log_set_writer_func(writer_func, std::ptr::null_mut(), None);
    }
}

/// Returns `true` for GDK cursor-theme warnings we want to suppress.
#[cfg(target_os = "linux")]
#[inline]
fn is_cursor_noise(msg: &str) -> bool {
    msg.contains("cursor") || msg.contains("Cursor") || msg.contains("Unable to load")
}
