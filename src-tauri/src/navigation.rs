//! The window only ever shows the app's own page.

use tauri::plugin::TauriPlugin;
use tauri::{Manager, Wry};

/// The app's own page: http://tauri.localhost on Windows, tauri://localhost
/// on Linux and macOS. Each is only the app on its own platforms: on Linux and
/// macOS http://tauri.localhost is a request to a local web server.
fn is_app_page(url: &tauri::Url) -> bool {
    let app = if cfg!(windows) {
        ("http", Some("tauri.localhost"))
    } else {
        ("tauri", Some("localhost"))
    };
    url.port().is_none() && (url.scheme(), url.host_str()) == app
}

/// Every navigation but to the app's own page (a link, a form or a refresh
/// tag in a document) is refused, so no other page can take the window and
/// its access to the app's commands. The dev server counts only when Tauri
/// serves the page from it, that is without the custom-protocol feature.
pub fn guard() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("navigation-guard")
        .on_navigation(|webview, url| {
            is_app_page(url)
                || (tauri::is_dev()
                    && webview
                        .config()
                        .build
                        .dev_url
                        .as_ref()
                        .is_some_and(|dev| dev.origin() == url.origin()))
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_app_page_is_an_app_page() {
        let (app, other_platform) = if cfg!(windows) {
            ("http://tauri.localhost", "tauri://localhost")
        } else {
            ("tauri://localhost", "http://tauri.localhost")
        };
        for ok in [
            app.to_string(),
            format!("{app}/"),
            format!("{app}/index.html#h-intro"),
        ] {
            assert!(is_app_page(&ok.parse().unwrap()), "{ok}");
        }
        for bad in [
            other_platform,
            "https://evil.example/",
            "http://evil.example/y",
            "http://tauri.localhost:8080/",
            "https://tauri.localhost/",
            "image://localhost/%2Ftmp%2Fx.svg",
            "http://image.localhost/x.svg",
            "file:///tmp/x.html",
            "tauri://evil.example/",
            "about:blank",
        ] {
            assert!(!is_app_page(&bad.parse().unwrap()), "{bad}");
        }
    }
}
