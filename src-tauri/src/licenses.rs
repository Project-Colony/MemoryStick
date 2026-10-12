//! The licences of the Rust libraries compiled into MemoryStick, which
//! their MIT, BSD, Apache-2.0 and similar terms require the executable to
//! carry. build.rs gathers them for the target being built.

const TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/rust-licenses.txt"));

/// The text Preferences > About shows under Third-party licences.
#[tauri::command]
pub fn third_party_licenses() -> &'static str {
    TEXT
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_the_libraries_memorystick_is_built_on() {
        for name in ["tauri", "comrak", "serde"] {
            let label = format!("{name}@");
            assert!(
                super::TEXT.lines().any(|line| line.starts_with(&label)),
                "{name} is missing from the third-party licences"
            );
        }
    }
}
