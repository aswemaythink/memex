//! Reads the log store's entries as docs for `show` (design §3.1 and §4.3): every non-dot `.md` file one folder below the store, labelled `stream:id`, with its frontmatter as fields and its title from its first line.
//! Cases: SHOW-LOG-01, SHOW-LOG-02, SHOW-JSON-03

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::collections::frontmatter::split;
use crate::corpus::Doc;
use crate::error::Error;
use crate::logs::write::title_of;
use crate::root::slash;
use crate::text;

/// Every entry in the store, as docs. A missing store has none. A store that cannot be read is an io error (exit 3).
pub fn load(store: &Path) -> Result<Vec<Doc>, Error> {
    let mut docs = Vec::new();
    for (stream, folder) in entries(store, true)? {
        for (file, path) in entries(&folder, false)? {
            docs.push(entry_doc(&stream, &file, &path)?);
        }
    }
    Ok(docs)
}

/// The non-dot entries of `dir` by name, sorted: its folders when `folders` is set, otherwise its `.md` files. A missing
/// `dir` has none.
fn entries(dir: &Path, folders: bool) -> Result<Vec<(String, PathBuf)>, Error> {
    let reader = match fs::read_dir(dir) {
        Ok(reader) => reader,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(failure) => return Err(io_error(dir, &failure)),
    };
    let mut found = Vec::new();
    for item in reader {
        let item = item.map_err(|failure| io_error(dir, &failure))?;
        let Some(name) = item.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let kind = item
            .file_type()
            .map_err(|failure| io_error(dir, &failure))?;
        let wanted = if folders {
            kind.is_dir()
        } else {
            kind.is_file() && name.ends_with(".md")
        };
        if wanted && !name.starts_with('.') {
            found.push((name, item.path()));
        }
    }
    found.sort();
    Ok(found)
}

/// One log entry as a doc: its frontmatter fields, its body as the text, and its title line.
fn entry_doc(stream: &str, file: &str, path: &Path) -> Result<Doc, Error> {
    let contents = text::read(path).map_err(|failure| io_error(path, &failure))?;
    let document = split(&contents);
    let id = file.strip_suffix(".md").unwrap_or(file).to_string();
    Ok(Doc {
        label: format!("{stream}:{id}"),
        collection: stream.to_string(),
        id,
        path: None,
        title: title_of(&document.body),
        text: document.body,
        fields: document.fields,
    })
}

/// A read failure, naming the path and the io kind and never the OS text (design §4.4).
fn io_error(path: &Path, failure: &io::Error) -> Error {
    Error::io(&format!("cannot read {}", slash(path)), failure)
}
