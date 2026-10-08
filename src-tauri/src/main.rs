// No console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod document;
mod markdown;
mod navigation;
mod preferences;

use std::path::PathBuf;

fn main() {
    // Answered before any window exists, so the release workflow can run the
    // binary on a runner without a display.
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }

    // Workarounds for webkit2gtk >= 2.44 on Linux (Wayland/XWayland):
    // - the DMA-BUF renderer allocates invalid GBM buffers, giving a blank page
    // - compositing mode is sometimes broken on Wayland
    // These must be set BEFORE the webview is initialised.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }

    // A file named on the command line (a double-click on Windows and
    // Linux) is the opened document; the page loads it once it is ready.
    let opened = document::Opened::default();
    if let Some(path) = std::env::args_os()
        .skip(1)
        .find(|a| !a.to_string_lossy().starts_with("--"))
    {
        opened.open(PathBuf::from(path));
    }

    tauri::Builder::default()
        .manage(opened)
        .plugin(navigation::guard())
        .register_asynchronous_uri_scheme_protocol("image", document::serve_image)
        .plugin(tauri_plugin_dialog::init())
        // Links are handled by one click handler in app.js, so the plugin's
        // own link script is left out.
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            document::open_document,
            document::load_file,
            preferences::save_preferences
        ])
        .on_window_event(document::on_window_event)
        // The window is built here rather than from tauri.conf.json alone,
        // which cannot place the web view's folder under Colony/MemoryStick.
        // The web view runs private, so it keeps nothing there between
        // runs: the preferences are a file of their own (preferences.rs).
        .setup(|app| {
            let config = app.config().app.windows[0].clone();
            let window = tauri::WebviewWindowBuilder::from_config(app, &config)?
                .incognito(true)
                .initialization_script(preferences::init_script());
            // macOS has no folder to choose: WebKit keeps its own.
            #[cfg(not(target_os = "macos"))]
            let window = match preferences::cache_dir() {
                Some(dir) => window.data_directory(dir.join("webview")),
                None => window,
            };
            window.build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start Tauri");
}
