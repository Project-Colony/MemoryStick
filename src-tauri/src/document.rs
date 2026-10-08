//! What MemoryStick reads: Markdown and text documents, and the images on
//! this device that they show.

use percent_encoding::percent_decode_str;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf, Prefix};
use tauri::http::{header::CONTENT_TYPE, Response, StatusCode};
use tauri::{Runtime, UriSchemeContext, UriSchemeResponder};

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

#[derive(Serialize)]
pub struct LoadedFile {
    html: String,
    file_path: String,
    file_name: String,
}

/// Reads a file. Refuses anything that is not a regular file (a device such
/// as /dev/zero, a pipe, a folder) and files larger than MAX_FILE_SIZE.
fn read_regular_file(path: &Path) -> Result<Vec<u8>, String> {
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
fn read_document(path: &Path) -> Result<String, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !EXTENSIONS
        .iter()
        .any(|known| known.eq_ignore_ascii_case(ext))
    {
        return Err("MemoryStick opens Markdown and text files only (.md, .markdown, .mdown, .mkd, .mkdn, .txt).".into());
    }
    String::from_utf8(read_regular_file(path)?).map_err(|_| "The file is not UTF-8 text.".into())
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
/// content type and bytes, or None for anything but an image on this device.
fn read_image(uri_path: &str) -> Option<(&'static str, Vec<u8>)> {
    let path = percent_decode_str(uri_path.strip_prefix('/')?)
        .decode_utf8()
        .ok()?;
    let path = Path::new(path.as_ref());
    let ext = path.extension()?.to_str()?;
    let (_, content_type) = IMAGE_TYPES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(ext))?;
    if !is_on_this_device(path) {
        return None;
    }
    Some((content_type, read_regular_file(path).ok()?))
}

#[tauri::command]
pub fn load_file(path: String) -> Result<LoadedFile, String> {
    let pb = PathBuf::from(&path);
    let html = crate::markdown::render(&read_document(&pb)?);
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

/// The image protocol: a document's images on this device, through app.js's
/// convertFileSrc(path, 'image'). Tauri's asset protocol is off: it checks
/// its scope only after a file system call on the path, which on Windows
/// connects to any network share a document names.
pub fn serve_image<R: Runtime>(
    _ctx: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let uri_path = request.uri().path().to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        let response = match read_image(&uri_path) {
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
    fn serves_only_images() {
        let dir = std::env::temp_dir().join(format!("memorystick-img-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        for name in ["a.png", "B.SVG", "c.md"] {
            std::fs::write(dir.join(name), name).unwrap();
        }
        let uri = |path: PathBuf| {
            let path = path.to_str().unwrap();
            format!(
                "/{}",
                percent_encoding::utf8_percent_encode(path, percent_encoding::NON_ALPHANUMERIC)
            )
        };

        assert_eq!(
            read_image(&uri(dir.join("a.png"))),
            Some(("image/png", b"a.png".to_vec()))
        );
        assert_eq!(
            read_image(&uri(dir.join("B.SVG"))),
            Some(("image/svg+xml", b"B.SVG".to_vec()))
        );
        // As a relative image ../a.png from a document in sub/
        assert!(read_image(&uri(dir.join("sub").join("..").join("a.png"))).is_some());
        assert_eq!(read_image(&uri(dir.join("c.md"))), None);
        assert_eq!(read_image(&uri(dir.join("missing.png"))), None);
        assert_eq!(read_image(&uri(PathBuf::from("a.png"))), None);

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
