// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Use the Adwaita cursor theme which has a complete cursor set, preventing
    // GDK "Unable to load <cursor> from the cursor theme" warnings from WebKitGTK.
    std::env::set_var("XCURSOR_THEME", "Adwaita");
    admin_portal_lib::run()
}
