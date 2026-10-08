//! Where MemoryStick keeps what it writes, and the preferences the page
//! reads before its first script runs.

use serde_json::{Map, Value};
use std::path::PathBuf;

/// The program's directory name under Colony/, spelled as the program is.
const PROGRAM: &str = "MemoryStick";

/// <config>/Colony/MemoryStick/, from the table in Project-Colony-Resources
/// design/filesystem.md: ~/.config on Linux, %LOCALAPPDATA% (never the
/// roaming AppData) on Windows, ~/Library/Application Support on macOS.
/// colony_ui::paths builds the same table, but that crate needs iced.
fn config_dir() -> Option<PathBuf> {
    Some(dirs::config_local_dir()?.join("Colony").join(PROGRAM))
}

/// <cache>/Colony/MemoryStick/: ~/.cache on Linux, ~/Library/Caches on
/// macOS. Windows has no cache root of its own, so there it is a cache\
/// folder next to the preferences, which clearing it then cannot delete.
pub fn cache_dir() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("Colony").join(PROGRAM);
    Some(if cfg!(windows) {
        dir.join("cache")
    } else {
        dir
    })
}

fn preferences_file() -> Option<PathBuf> {
    Some(config_dir()?.join("preferences").join("preferences.json"))
}

/// The saved preferences. A missing or unreadable file gives none, and the
/// page falls back to its defaults.
fn load() -> Map<String, Value> {
    preferences_file()
        .and_then(|path| std::fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Writes the page's preferences, whole. The page owns their meaning; this
/// only keeps them. Written to a temporary file first, so a crash midway
/// leaves the previous preferences, not half of them.
#[tauri::command]
pub async fn save_preferences(preferences: Map<String, Value>) -> Result<(), String> {
    let path = preferences_file().ok_or("no configuration folder on this system")?;
    let tmp = path.with_extension("json.tmp");
    std::fs::create_dir_all(path.parent().unwrap_or(&path))
        .and_then(|()| std::fs::write(&tmp, Value::Object(preferences).to_string()))
        .and_then(|()| std::fs::rename(&tmp, &path))
        .map_err(|e| e.to_string())
}

/// Runs before any script of the page: window.__MEMORYSTICK__ holds the
/// version and the saved preferences, so the first paint is already in the
/// user's theme.
pub fn init_script() -> String {
    format!(
        "window.__MEMORYSTICK__ = {{ version: {}, preferences: {} }};",
        Value::from(env!("CARGO_PKG_VERSION")),
        Value::Object(load()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_folder_is_colony_then_memorystick() {
        for dir in [config_dir().unwrap(), cache_dir().unwrap()] {
            let tail: Vec<_> = dir
                .components()
                .rev()
                .skip(usize::from(cfg!(windows) && dir.ends_with("cache")))
                .take(2)
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            assert_eq!(tail, ["MemoryStick", "Colony"], "{dir:?}");
        }
        // The cache never is, nor holds, the preferences.
        assert!(!preferences_file()
            .unwrap()
            .starts_with(cache_dir().unwrap()));
    }
}
