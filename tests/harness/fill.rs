//! Makes a case's `fill` files: NUL bytes at their real lengths, so that no case commits a large file.
//! A fill path must stay inside the sandbox, and a file that already exists is never replaced.
//! Cases: all (runner)

use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Component, Path};

use super::FillStep;

/// Makes each fill file under `root`: its parent folders first, then the file, with its length set. The first failure
/// stops the list and names its path.
pub fn create_files(root: &Path, fills: &[FillStep]) -> Result<(), String> {
    for step in fills {
        create_file(root, step)?;
    }
    Ok(())
}

/// Makes one fill file. Its path is checked first, so nothing outside the sandbox is touched.
fn create_file(root: &Path, step: &FillStep) -> Result<(), String> {
    if !stays_inside(&step.path) {
        return Err(format!(
            "fill path {:?} is not a path inside the sandbox: write it relative to the sandbox, such as repo/.memex/logs/impl/x.md, with / between names, no .., no leading / and no drive",
            step.path
        ));
    }
    let path = root.join(&step.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| failure("cannot create the folders of fill", &step.path, &e))?;
    }
    let file = open_new(&path, &step.path)?;
    file.set_len(step.bytes)
        .map_err(|e| failure("cannot set the length of fill", &step.path, &e))
}

/// Opens a file that must not exist yet. A file the fixture or an earlier fill entry made is refused, never replaced.
fn open_new(path: &Path, shown: &str) -> Result<fs::File, String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| match e.kind() {
            io::ErrorKind::AlreadyExists => format!(
                "fill path {shown:?} already exists in the sandbox: the fixture or an earlier fill entry holds it; delete one of them"
            ),
            _ => failure("cannot create fill", shown, &e),
        })
}

/// True when `path` names a place inside the sandbox: relative, made of plain names with `/` between them, and with no
/// `..`, no drive and no `:` or `\`, so that it means the same on every OS.
fn stays_inside(path: &str) -> bool {
    !path.is_empty()
        && !path.ends_with('/')
        && !path.contains([':', '\\'])
        && Path::new(path)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

/// A failure message that names what failed, the path and the io::ErrorKind, never the raw OS text.
fn failure(what: &str, path: &str, error: &io::Error) -> String {
    format!("{what} {path:?} ({:?})", error.kind())
}
