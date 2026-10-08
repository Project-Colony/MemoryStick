// No console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod markdown;

use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};

/// The file types MemoryStick opens. The Open dialog offers the same list.
const EXTENSIONS: [&str; 6] = ["md", "markdown", "mdown", "mkd", "mkdn", "txt"];
/// Larger files are refused instead of being read into memory.
const MAX_FILE_SIZE: u64 = 32 * 1024 * 1024;

#[derive(Serialize)]
struct LoadedFile {
    html: String,
    file_path: String,
    file_name: String,
}

/// Reads a document. Refuses other file types, anything that is not a
/// regular file (a device such as /dev/zero, a pipe, a folder) and files
/// larger than MAX_FILE_SIZE.
fn read_document(path: &Path) -> Result<String, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !EXTENSIONS
        .iter()
        .any(|known| known.eq_ignore_ascii_case(ext))
    {
        return Err("MemoryStick opens Markdown and text files only (.md, .markdown, .mdown, .mkd, .mkdn, .txt).".into());
    }
    let too_large = || format!("The file is larger than {} MiB.", MAX_FILE_SIZE >> 20);
    let cannot_read = |e: std::io::Error| format!("Cannot read the file: {e}");
    let meta = std::fs::metadata(path).map_err(cannot_read)?;
    if !meta.is_file() {
        return Err("This is not a regular file.".into());
    }
    if meta.len() > MAX_FILE_SIZE {
        return Err(too_large());
    }
    // Bounded too, in case the file grew since its size was read.
    let mut content = String::new();
    File::open(path)
        .and_then(|f| f.take(MAX_FILE_SIZE + 1).read_to_string(&mut content))
        .map_err(cannot_read)?;
    if content.len() as u64 > MAX_FILE_SIZE {
        return Err(too_large());
    }
    Ok(content)
}

#[tauri::command]
fn load_file(path: String) -> Result<LoadedFile, String> {
    let pb = PathBuf::from(&path);
    let html = markdown::render(&read_document(&pb)?);
    let file_name = pb
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(LoadedFile {
        html,
        file_path: path,
        file_name,
    })
}

/// The app's own page: tauri://localhost on Linux and macOS,
/// http://tauri.localhost on Windows.
fn is_app_page(url: &tauri::Url) -> bool {
    url.port().is_none()
        && matches!(
            (url.scheme(), url.host_str()),
            ("tauri", Some("localhost")) | ("http", Some("tauri.localhost"))
        )
}

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
        // The window only ever shows the app's own page. Every other
        // navigation (a link, a form or a refresh tag in a document) is
        // refused, so no other page can take the window and its access to
        // the app's commands.
        .plugin(
            tauri::plugin::Builder::<tauri::Wry>::new("navigation-guard")
                .on_navigation(|webview, url| {
                    is_app_page(url)
                        || (cfg!(debug_assertions)
                            && webview
                                .config()
                                .build
                                .dev_url
                                .as_ref()
                                .is_some_and(|dev| dev.origin() == url.origin()))
                })
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        // Links are handled by one click handler in app.js, so the plugin's
        // own link script is left out.
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![load_file])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_app_page_is_an_app_page() {
        for ok in [
            "tauri://localhost",
            "tauri://localhost/index.html#h-intro",
            "http://tauri.localhost/",
        ] {
            assert!(is_app_page(&ok.parse().unwrap()), "{ok}");
        }
        for bad in [
            "https://evil.example/",
            "http://evil.example/y",
            "http://tauri.localhost:8080/",
            "https://tauri.localhost/",
            "asset://localhost/%2Ftmp%2Fx.html",
            "http://asset.localhost/x.html",
            "file:///tmp/x.html",
            "tauri://evil.example/",
            "about:blank",
        ] {
            assert!(!is_app_page(&bad.parse().unwrap()), "{bad}");
        }
    }

    #[test]
    fn reads_only_small_documents_of_known_types() {
        let dir = std::env::temp_dir().join(format!("memorystick-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = |name: &str, len: u64| {
            let path = dir.join(name);
            let f = File::create(&path).unwrap();
            f.set_len(len).unwrap(); // sparse: no disk space used
            path
        };

        for ok in [
            "a.md",
            "B.MD",
            "c.markdown",
            "d.mdown",
            "e.mkd",
            "f.mkdn",
            "g.txt",
        ] {
            assert_eq!(read_document(&file(ok, 3)), Ok("\0\0\0".into()), "{ok}");
        }
        for refused in [
            "id_ed25519",
            "config",
            ".md",
            "x.md.exe",
            "x.html",
            "x.json",
        ] {
            assert!(read_document(&file(refused, 3)).is_err(), "{refused}");
        }
        assert!(read_document(&file("big.md", MAX_FILE_SIZE + 1)).is_err());
        std::fs::create_dir_all(dir.join("folder.md")).unwrap();
        assert!(read_document(&dir.join("folder.md")).is_err());
        assert!(read_document(&dir.join("missing.md")).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/dev/zero", dir.join("zero.md")).unwrap();
            assert!(read_document(&dir.join("zero.md")).is_err());
        }

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
