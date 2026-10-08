//! What MemoryStick reads: the document the user opened, and the images on
//! this device, in that document's folder, that it shows. Nothing else.

use percent_encoding::percent_decode_str;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf, Prefix};
use std::sync::Mutex;
use tauri::http::{header::CONTENT_TYPE, Response, StatusCode};
use tauri::{
    DragDropEvent, Emitter, Manager, Runtime, State, UriSchemeContext, UriSchemeResponder, Window,
    WindowEvent,
};
use tauri_plugin_dialog::DialogExt;

/// The file types MemoryStick opens. The Open dialog offers the same list.
const EXTENSIONS: [&str; 6] = ["md", "markdown", "mdown", "mkd", "mkdn", "txt"];
/// The images a document can show from the device, and their content types.
const IMAGE_TYPES: [(&str, &str); 9] = [
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("avif", "image/avif"),
    ("svg", "image/svg+xml"),
    ("bmp", "image/bmp"),
    ("ico", "image/x-icon"),
];
/// Larger files are refused instead of being read into memory.
const MAX_FILE_SIZE: u64 = 32 * 1024 * 1024;

/// The documents the user opened: through the Open dialog, by dropping one
/// on the window, or by naming one on the command line. The page cannot
/// name a file itself, so a document can make MemoryStick read neither
/// another document nor anything else.
#[derive(Default)]
pub struct Opened(Mutex<Documents>);

#[derive(Default)]
struct Documents {
    /// The document on screen.
    shown: Option<PathBuf>,
    /// One the user just opened, until it is read.
    next: Option<PathBuf>,
}

impl Opened {
    pub fn open(&self, path: PathBuf) {
        self.lock().next = Some(path);
    }

    /// Reads the document the user just opened, or the one on screen again.
    /// A file that cannot be read leaves the one on screen in place.
    fn read(&self) -> Result<Option<LoadedFile>, Failure> {
        let path = {
            let mut docs = self.lock();
            docs.next.take().or_else(|| docs.shown.clone())
        };
        let Some(path) = path else {
            return Ok(None);
        };
        let loaded = load(&path)?;
        self.lock().shown = Some(path);
        Ok(Some(loaded))
    }

    /// The folder of the document on screen.
    fn folder(&self) -> Option<PathBuf> {
        Some(self.lock().shown.as_ref()?.parent()?.to_path_buf())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Documents> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Why a document could not be read, for the page to say in the user's
/// language: an error_* key of dist/i18n/, and what fills in its {detail}.
#[derive(Debug, PartialEq, Serialize)]
pub struct Failure {
    key: &'static str,
    detail: Option<String>,
}

impl Failure {
    fn new(key: &'static str) -> Self {
        Failure { key, detail: None }
    }

    fn with(key: &'static str, detail: impl ToString) -> Self {
        Failure {
            key,
            detail: Some(detail.to_string()),
        }
    }
}

#[derive(Serialize)]
pub struct LoadedFile {
    html: String,
    file_path: String,
    file_name: String,
}

/// Reads a file. Refuses anything that is not a regular file (a device such
/// as /dev/zero, a pipe, a folder) and files larger than MAX_FILE_SIZE.
fn read_regular_file(path: &Path) -> Result<Vec<u8>, Failure> {
    let too_large = || Failure::with("error_too_large", MAX_FILE_SIZE >> 20);
    let cannot_read = |e: std::io::Error| Failure::with("error_cannot_read", e);
    let meta = std::fs::metadata(path).map_err(cannot_read)?;
    if !meta.is_file() {
        return Err(Failure::new("error_not_a_file"));
    }
    if meta.len() > MAX_FILE_SIZE {
        return Err(too_large());
    }
    // Bounded too, in case the file grew since its size was read.
    let mut content = Vec::new();
    File::open(path)
        .and_then(|f| f.take(MAX_FILE_SIZE + 1).read_to_end(&mut content))
        .map_err(cannot_read)?;
    if content.len() as u64 > MAX_FILE_SIZE {
        return Err(too_large());
    }
    Ok(content)
}

/// Reads a document: a Markdown or text file, as read_regular_file allows.
fn read_document(path: &Path) -> Result<String, Failure> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !EXTENSIONS
        .iter()
        .any(|known| known.eq_ignore_ascii_case(ext))
    {
        return Err(Failure::new("error_not_markdown"));
    }
    String::from_utf8(read_regular_file(path)?).map_err(|_| Failure::new("error_not_utf8"))
}

/// Whether a path names a file on this device. On Windows that means a path
/// on a drive (C:\...): a network share (\\server\share\..., also written
/// //server/share/...) or a device (\\.\...) is refused before any file
/// system call, since opening one connects to the server and sends it the
/// user's Windows credentials.
fn is_on_this_device(path: &Path) -> bool {
    path.is_absolute()
        && match path.components().next() {
            Some(Component::Prefix(prefix)) => {
                matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
            }
            _ => !cfg!(windows),
        }
}

/// Reads an image a document shows, for the image protocol. `uri_path` is
/// the path of image://localhost/<path> (http://image.localhost/<path> on
/// Windows), where app.js puts the percent-encoded file path. Returns its
/// content type and bytes, or None for anything but an image on this device
/// inside `folder`, the opened document's folder or one below it.
fn read_image(folder: &Path, uri_path: &str) -> Option<(&'static str, Vec<u8>)> {
    let path = percent_decode_str(uri_path.strip_prefix('/')?)
        .decode_utf8()
        .ok()?;
    let path = Path::new(path.as_ref());
    let ext = path.extension()?.to_str()?;
    let (_, content_type) = IMAGE_TYPES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(ext))?;
    // Checked before any file system call, which canonicalize is.
    if !is_on_this_device(path) {
        return None;
    }
    // Resolves .. and links, so neither can lead outside the folder.
    let path = path.canonicalize().ok()?;
    if !path.starts_with(folder.canonicalize().ok()?) {
        return None;
    }
    Some((content_type, read_regular_file(&path).ok()?))
}

fn load(path: &Path) -> Result<LoadedFile, Failure> {
    Ok(LoadedFile {
        html: crate::markdown::render(&read_document(path)?),
        file_path: path.to_string_lossy().into_owned(),
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
    })
}

/// Shows the Open dialog and opens the file the user picks. None when the
/// dialog is cancelled. `all_files` names the dialog's catch-all filter,
/// kept because GTK matches the Markdown patterns case-sensitively, so on
/// Linux README.MD only shows under it.
#[tauri::command]
pub async fn open_document<R: Runtime>(
    window: Window<R>,
    opened: State<'_, Opened>,
    all_files: String,
) -> Result<Option<LoadedFile>, Failure> {
    let dialog = window
        .dialog()
        .file()
        .add_filter("Markdown", &EXTENSIONS)
        .add_filter(all_files, &["*"]);
    #[cfg(any(windows, target_os = "macos"))]
    let dialog = dialog.set_parent(&window);
    let Some(path) = dialog.blocking_pick_file().and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    opened.open(path);
    opened.read()
}

/// Reads the document a drop or the command line opened, or the one on
/// screen again (Ctrl+R). None when no document is open.
#[tauri::command]
pub async fn load_file(opened: State<'_, Opened>) -> Result<Option<LoadedFile>, Failure> {
    opened.read()
}

/// A file dropped on the window becomes the opened document, and the page
/// is told to load it.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
        if let Some(path) = paths.first() {
            window.state::<Opened>().open(path.clone());
            let _ = window.emit("document-opened", ());
        }
    }
}

/// The image protocol: a document's images on this device, through app.js's
/// convertFileSrc(path, 'image'). Tauri's asset protocol is off: it checks
/// its scope only after a file system call on the path, which on Windows
/// connects to any network share a document names.
pub fn serve_image<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let uri_path = request.uri().path().to_owned();
    let folder = ctx.app_handle().state::<Opened>().folder();
    tauri::async_runtime::spawn_blocking(move || {
        let response = match folder.and_then(|folder| read_image(&folder, &uri_path)) {
            Some((content_type, bytes)) => Response::builder()
                .header(CONTENT_TYPE, content_type)
                .body(bytes),
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Vec::new()),
        };
        responder.respond(response.expect("a valid static response"));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn only_paths_on_this_device_are_on_this_device() {
        let (local, elsewhere): (&[&str], &[&str]) = if cfg!(windows) {
            (
                &[r"C:\docs\a.png", "C:/docs/a.png", r"\\?\C:\docs\a.png"],
                &[
                    r"\\server\share\a.png",
                    "//server/share/a.png",
                    r"\\?\UNC\server\share\a.png",
                    r"\\.\pipe\a.png",
                    r"\??\UNC\server\share\a.png",
                    r"\docs\a.png",
                    "C:a.png",
                    "a.png",
                ],
            )
        } else {
            (&["/home/a.png"], &["a.png", "../a.png"])
        };
        for path in local {
            assert!(is_on_this_device(Path::new(path)), "{path}");
        }
        for path in elsewhere {
            assert!(!is_on_this_device(Path::new(path)), "{path}");
        }
    }

    #[test]
    fn a_document_that_cannot_be_read_leaves_the_shown_one() {
        let dir = std::env::temp_dir().join(format!("memorystick-open-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.md"), "# A").unwrap();
        let opened = Opened::default();

        assert!(opened.read().unwrap().is_none(), "nothing open yet");
        opened.open(dir.join("a.md"));
        assert!(opened.read().unwrap().is_some());
        opened.open(dir.join("missing.md"));
        assert!(opened.read().is_err());
        // Still a.md: reloading shows it, and its folder serves images.
        assert_eq!(opened.read().unwrap().unwrap().file_name, "a.md");
        assert_eq!(opened.folder(), Some(dir.clone()));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn serves_only_images_in_the_document_folder() {
        let dir = std::env::temp_dir().join(format!("memorystick-img-{}", std::process::id()));
        let folder = dir.join("doc");
        std::fs::create_dir_all(folder.join("sub")).unwrap();
        for name in ["a.png", "B.SVG", "c.md", "sub/inner.png", "../outside.png"] {
            std::fs::write(folder.join(name), name).unwrap();
        }
        let uri = |path: PathBuf| {
            let path = path.to_str().unwrap();
            format!(
                "/{}",
                percent_encoding::utf8_percent_encode(path, percent_encoding::NON_ALPHANUMERIC)
            )
        };
        let read = |path: PathBuf| read_image(&folder, &uri(path));

        assert_eq!(
            read(folder.join("a.png")),
            Some(("image/png", b"a.png".to_vec()))
        );
        assert_eq!(
            read(folder.join("B.SVG")),
            Some(("image/svg+xml", b"B.SVG".to_vec()))
        );
        assert!(read(folder.join("sub").join("inner.png")).is_some());
        // ../a.png from a page in sub/ is still inside the folder.
        assert!(read(folder.join("sub").join("..").join("a.png")).is_some());
        // Outside the folder, by .. or by its own path.
        assert_eq!(read(folder.join("..").join("outside.png")), None);
        assert_eq!(read(dir.join("outside.png")), None);
        assert_eq!(read(folder.join("c.md")), None);
        assert_eq!(read(folder.join("missing.png")), None);
        assert_eq!(read(PathBuf::from("a.png")), None);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.join("outside.png"), folder.join("link.png")).unwrap();
            assert_eq!(read(folder.join("link.png")), None);
        }

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
