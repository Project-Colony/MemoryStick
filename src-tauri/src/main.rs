// No console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod markdown;

use serde::Serialize;
use std::path::PathBuf;
use tauri::{Emitter, Manager};

#[derive(Serialize)]
struct LoadedFile {
    content: String,
    html: String,
    file_path: String,
    file_name: String,
}

#[tauri::command]
fn render_markdown(content: String) -> String {
    markdown::render(&content)
}

#[tauri::command]
fn load_file(path: String) -> Result<LoadedFile, String> {
    let pb = PathBuf::from(&path);
    let content = std::fs::read_to_string(&pb)
        .map_err(|e| format!("Cannot read the file: {e}"))?;
    let html = markdown::render(&content);
    let file_name = pb
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(LoadedFile {
        content,
        html,
        file_path: path,
        file_name,
    })
}

fn main() {
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![render_markdown, load_file])
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
