use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn main() {
    write_rust_licenses();

    // Declaring the app's commands turns on Tauri's permission check for
    // them, so they answer only the pages that capabilities/ allows: the
    // app's own page, never a page from anywhere else.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "open_document",
            "load_file",
            "save_preferences",
            "third_party_licenses",
        ]),
    ))
    .expect("tauri-build failed");
}

/// File names, lowercased, that hold a library's licence or notices.
const LICENCE_FILES: [&str; 6] = [
    "license",
    "licence",
    "unlicense",
    "copying",
    "copyright",
    "notice",
];

/// Writes $OUT_DIR/rust-licenses.txt for src/licenses.rs: the licence
/// texts of every library linked into the executable for this target, which
/// MIT, BSD, Apache-2.0 and the like require a binary to carry. The texts
/// come from the packages Cargo already downloaded to build them, so this
/// never reaches the network.
fn write_rust_licenses() {
    println!("cargo:rerun-if-changed=Cargo.lock");
    let var = |name: &str| env::var(name).unwrap_or_else(|_| panic!("{name} is not set"));
    let target = var("TARGET");

    let output = Command::new(var("CARGO"))
        .args(["metadata", "--format-version", "1", "--locked", "--offline"])
        .args(["--filter-platform", &target, "--manifest-path"])
        .arg(Path::new(&var("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .output()
        .expect("cannot run cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata printed no JSON");

    let text = rust_licenses(&metadata, &var("CARGO_PKG_VERSION"), &target);
    fs::write(Path::new(&var("OUT_DIR")).join("rust-licenses.txt"), text)
        .expect("cannot write rust-licenses.txt");
}

/// The text: one block per distinct licence text, headed by the libraries
/// that ship it, then the libraries that ship no licence file at all.
fn rust_licenses(metadata: &Value, version: &str, target: &str) -> String {
    let packages: HashMap<&str, &Value> = slice(&metadata["packages"])
        .iter()
        .map(|p| (p["id"].as_str().unwrap_or_default(), p))
        .collect();
    let nodes: HashMap<&str, &Value> = slice(&metadata["resolve"]["nodes"])
        .iter()
        .map(|n| (n["id"].as_str().unwrap_or_default(), n))
        .collect();

    // Normal dependencies only, followed from MemoryStick: build and dev
    // dependencies run on the build machine and are not linked in.
    let mut linked = BTreeSet::new();
    let mut todo = vec![metadata["resolve"]["root"]
        .as_str()
        .expect("cargo metadata names no root package")];
    while let Some(id) = todo.pop() {
        for dep in nodes.get(id).map_or(&[][..], |n| slice(&n["deps"])) {
            let normal = slice(&dep["dep_kinds"]).iter().any(|k| k["kind"].is_null());
            let pkg = dep["pkg"].as_str().unwrap_or_default();
            if normal && linked.insert(pkg) {
                todo.push(pkg);
            }
        }
    }

    // Keyed by the words alone: copies of one licence that differ only in
    // line breaks, indentation or trailing spaces count as one text, shown
    // as the first library to ship it does.
    let mut by_text: BTreeMap<String, (String, BTreeSet<String>)> = BTreeMap::new();
    let mut without_file = BTreeSet::new();
    for id in linked {
        let package = packages[id];
        let str_of = |key: &str| package[key].as_str().unwrap_or_default();
        let label = match str_of("license") {
            "" => format!("{}@{}", str_of("name"), str_of("version")),
            expr => format!("{}@{} ({expr})", str_of("name"), str_of("version")),
        };
        let texts: BTreeSet<String> =
            licence_files(Path::new(str_of("manifest_path")), str_of("license_file"))
                .iter()
                .filter_map(|file| fs::read(file).ok())
                .map(|bytes| normalise(&String::from_utf8_lossy(&bytes)))
                .filter(|text| !text.is_empty())
                .collect();
        if texts.is_empty() {
            without_file.insert(label);
        } else {
            for text in texts {
                let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
                let (_, libs) = by_text
                    .entry(words)
                    .or_insert_with(|| (text, BTreeSet::new()));
                libs.insert(label.clone());
            }
        }
    }

    let mut blocks: Vec<_> = by_text
        .into_values()
        .map(|(text, libs)| (libs, text))
        .collect();
    blocks.sort();
    let rule = |c: &str| c.repeat(72);
    let mut out = format!(
        "The Rust libraries compiled into MemoryStick {version} for {target}.\n\
         Each list of libraries is followed by the licence text they ship.\n"
    );
    for (libs, text) in blocks {
        out += &format!("\n{}\n", rule("="));
        for lib in libs {
            out += &format!("{lib}\n");
        }
        out += &format!("{}\n\n{text}\n", rule("-"));
    }
    if !without_file.is_empty() {
        out += &format!(
            "\n{}\nThese libraries ship no licence file. Each is under the licence its\n\
             manifest names, whose standard text is at https://spdx.org/licenses/.\n{}\n",
            rule("="),
            rule("-")
        );
        for lib in without_file {
            out += &format!("{lib}\n");
        }
    }
    out
}

/// The licence files at the top of a package, plus its declared
/// license-file.
fn licence_files(manifest: &Path, declared: &str) -> BTreeSet<PathBuf> {
    let dir = manifest.parent().unwrap_or(manifest);
    let mut files: BTreeSet<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            path.is_file() && LICENCE_FILES.iter().any(|prefix| name.starts_with(prefix))
        })
        .collect();
    if !declared.is_empty() {
        files.insert(dir.join(declared));
    }
    files
}

/// The text without carriage returns, trailing spaces or blank lines at
/// either end.
fn normalise(text: &str) -> String {
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    lines.join("\n").trim_matches('\n').to_owned()
}

fn slice(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}
