fn main() {
    // Declaring the app's commands turns on Tauri's permission check for
    // them, so they answer only the pages that capabilities/ allows: the
    // app's own page, never a page from anywhere else.
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&["open_document", "load_file"]),
        ),
    )
    .expect("tauri-build failed");
}
