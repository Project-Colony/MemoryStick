fn main() {
    // Declaring the app's command turns on Tauri's permission check for it,
    // so load_file answers only the pages that capabilities/ allows: the
    // app's own page, never a page from anywhere else.
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["load_file"])),
    )
    .expect("tauri-build failed");
}
