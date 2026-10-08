// No console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod document;
mod markdown;
mod navigation;

use std::path::PathBuf;
use tauri::{Emitter, Manager};

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

    tauri::Builder::default()
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
        .invoke_handler(tauri::generate_handler![document::load_file])
        .setup(|app| {
            // A file passed as an argument (double-click on Windows/Linux)
            let args: Vec<String> = std::env::args().skip(1).collect();
            if let Some(file_arg) = args.iter().find(|a| !a.starts_with("--")) {
                let pb = PathBuf::from(file_arg);
                if pb.is_file() {
                    if let Some(window) = app.get_webview_window("main") {
                        let path_str = pb.to_string_lossy().to_string();
                        // Give the frontend a moment to get ready
                        let win_clone = window.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(400));
                            let _ = win_clone.emit("open-file-path", path_str);
                        });
                    }
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start Tauri");
}
