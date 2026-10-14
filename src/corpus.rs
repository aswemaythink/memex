//! Loads the read model (design §3.1 and §3.6): the walked entries and the docs built from them.
//! The docs are READMEs, headers, CASES.md, manifests and logs, with the label map and the folder index.
//! Cases: SHOW-FILE-01, SHOW-FILE-02, SHOW-DIR-01, SHOW-DIR-02, SHOW-ROOT-01, SHOW-ROOT-02, SHOW-LOG-01, SHOW-LOG-02, SHOW-LINE-01, INT-NAME-01, INT-NAME-02, INT-CHAIN-01, INT-BOM-01, MAN-LOAD-02, REF-AMBIG-02, REF-AMBIG-03, REF-RESERVED-03, REF-RESERVED-04

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;

use crate::cite;
use crate::collections::frontmatter::Value;
use crate::collections::manifest;
use crate::error::Error;
use crate::intent::{self, cases, header, readme};
use crate::logs::read::load as load_logs;
use crate::root::slash;
use crate::text;
use crate::walk::{self, Entry};

pub use crate::root::root_of;

/// The most bytes of a file that header detection reads (design §2.1).
const HEAD_BYTES: u64 = 8 * 1024;

/// One indexed doc: an intent doc labelled by its path, or a collection doc labelled `collection:id` (design §3.1 and §3.6).
#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    /// The label `show` takes: a root-relative path for an intent doc, `collection:id` for any other.
    pub label: String,
    /// The collection: `readmes`, `headers`, `cases`, a manifest's name, or the stream of a log.
    pub collection: String,
    /// The id within the collection: the path of an intent doc, the file stem of a log.
    pub id: String,
    /// The root-relative path of an intent doc; None for a log.
    pub path: Option<String>,
    /// The one-liner: a README's or a header's first sentence, or a log's title line.
    pub title: String,
    /// The text `show` prints: a README in full, a header, or a log body.
    pub text: String,
    /// The frontmatter fields, in the order the file gives them.
    pub fields: Vec<(String, Value)>,
}

/// Whether a walked entry is a folder or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A folder.
    Dir,
    /// A file.
    File,
}

/// One child of a folder: a subfolder or a file, with its one-liner when it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Child {
    /// The last path component, as it is on disk.
    pub name: String,
    /// The root-relative path.
    pub rel: String,
    /// Whether the child is a folder or a file.
    pub kind: Kind,
    /// The one-liner of a folder's README, or of a file's header or README; None when there is none.
    pub title: Option<String>,
}

/// The read model: every doc by label, every walked entry, the children of every folder, and the log streams.
pub struct Corpus {
    /// The root the intent layer is read from.
    pub root: PathBuf,
    /// Every doc by label, in load order. A label holds more than one doc when two files declare the same id.
    docs: BTreeMap<String, Vec<Doc>>,
    kinds: BTreeMap<String, Kind>,
    children: BTreeMap<String, Vec<Child>>,
    /// The streams of the log store, which `logs:` reads across (design §3.1).
    streams: BTreeSet<String>,
    /// The citations of every doc and the backlinks they give, built once when the corpus loads (design §2.3).
    citations: cite::Index,
}

impl Corpus {
    /// Reads the intent layer under `root` and, when there is a store, the logs in it. A missing store has no logs.
    pub fn load(root: &Path, store: Option<&Path>) -> Result<Corpus, Error> {
        let mut corpus = Corpus::intent(root)?;
        corpus.finish(store)?;
        Ok(corpus)
    }

    /// Like `load`, with the store found by `find_store`, which runs on its own thread while the intent layer is read, since
    /// finding the store may run git. Returns the store too. An error in finding the store is reported before a read error.
    pub fn load_finding<F>(root: &Path, find_store: F) -> Result<(Corpus, Option<PathBuf>), Error>
    where
        F: FnOnce() -> Result<Option<PathBuf>, Error> + Send,
    {
        let (intent, store) = thread::scope(|scope| {
            let finder = scope.spawn(find_store);
            let intent = Corpus::intent(root);
            let store = finder
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
            (intent, store)
        });
        let store = store?;
        let mut corpus = intent?;
        corpus.finish(store.as_deref())?;
        Ok((corpus, store))
    }

    /// The intent layer under `root`: every walked entry, the docs of its files, and the docs of its manifests.
    fn intent(root: &Path) -> Result<Corpus, Error> {
        let mut corpus = Corpus {
            root: root.to_path_buf(),
            docs: BTreeMap::new(),
            kinds: BTreeMap::from([(String::new(), Kind::Dir)]),
            children: BTreeMap::new(),
            streams: BTreeSet::new(),
            citations: cite::Index::default(),
        };
        for entry in walk::walk(root) {
            let kind = if entry.is_dir { Kind::Dir } else { Kind::File };
            corpus.kinds.insert(entry.rel.clone(), kind);
            if !entry.is_dir {
                for doc in file_docs(&entry)? {
                    corpus.insert(doc);
                }
            }
        }
        let mut warnings = Vec::new();
        for found in manifest::load(root, &mut warnings) {
            for doc in manifest::docs(&found, root, &mut warnings) {
                corpus.insert(doc);
            }
        }
        warn(&warnings);
        Ok(corpus)
    }

    /// Adds the logs of the store, when there is one, then builds the folder index and the citations.
    fn finish(&mut self, store: Option<&Path>) -> Result<(), Error> {
        if let Some(store) = store {
            let logs = load_logs(store)?;
            self.streams = logs.iter().map(|doc| doc.collection.clone()).collect();
            for doc in logs {
                self.insert(doc);
            }
        }
        self.index_children();
        self.citations = cite::Index::build(self);
        Ok(())
    }

    /// The first doc with this label, which titles and text are read from. `labelled` gives every doc with it.
    pub fn doc(&self, label: &str) -> Option<&Doc> {
        self.labelled(label).first()
    }

    /// Every doc with this label, in load order: one, or several when two files declare the same id (design §4.2).
    pub fn labelled(&self, label: &str) -> &[Doc] {
        match self.docs.get(label) {
            Some(docs) => docs,
            None => &[],
        }
    }

    /// Every doc, in label order.
    pub fn all_docs(&self) -> impl Iterator<Item = &Doc> {
        self.docs.values().flatten()
    }

    /// The first doc of each label, in label order: the docs that `doc` finds, which `search` ranks.
    pub fn first_docs(&self) -> impl Iterator<Item = &Doc> {
        self.docs.values().filter_map(|docs| docs.first())
    }

    /// True for a log entry: a doc with no path whose collection is a stream of the log store (design §3.1, `logs`).
    pub fn is_log(&self, doc: &Doc) -> bool {
        doc.path.is_none() && self.streams.contains(&doc.collection)
    }

    /// The root-relative paths of the walked files, in label order.
    pub fn files(&self) -> impl Iterator<Item = &str> {
        self.kinds
            .iter()
            .filter(|(_, kind)| **kind == Kind::File)
            .map(|(rel, _)| rel.as_str())
    }

    /// What the walk found at this root-relative path: a folder, a file, or nothing.
    pub fn kind(&self, rel: &str) -> Option<Kind> {
        self.kinds.get(rel).copied()
    }

    /// The children of a folder, sorted by name. Empty for a file and for a path the walk did not find.
    pub fn children(&self, rel: &str) -> &[Child] {
        match self.children.get(rel) {
            Some(list) => list,
            None => &[],
        }
    }

    /// The citations of every doc and the backlinks they give (design §2.3).
    pub fn citations(&self) -> &cite::Index {
        &self.citations
    }

    /// Adds a doc under its label. Every doc is kept, so a label that two files declare is seen as ambiguous, not guessed.
    fn insert(&mut self, doc: Doc) {
        self.docs.entry(doc.label.clone()).or_default().push(doc);
    }

    /// Builds the folder index: every walked entry becomes a child of its parent folder.
    fn index_children(&mut self) {
        let mut index: BTreeMap<String, Vec<Child>> = BTreeMap::new();
        for (rel, kind) in &self.kinds {
            let Some(parent) = parent_of(rel) else {
                continue;
            };
            let child = Child {
                name: name_of(rel),
                rel: rel.clone(),
                kind: *kind,
                title: self.title_of(rel, *kind),
            };
            index.entry(parent.to_string()).or_default().push(child);
        }
        self.children = index;
    }

    /// The one-liner of a walked entry: its README's for a folder, its own doc's for a file.
    fn title_of(&self, rel: &str, kind: Kind) -> Option<String> {
        let label = match kind {
            Kind::Dir => readme_label(rel),
            Kind::File => rel.to_string(),
        };
        self.doc(&label).map(|doc| doc.title.clone())
    }
}

/// The label of a root-relative path: `.` for the root, the path itself otherwise.
pub fn label_of(rel: &str) -> String {
    if rel.is_empty() {
        ".".to_string()
    } else {
        rel.to_string()
    }
}

/// The parent folder of a root-relative path: `""` for a top-level entry, and None for the root itself.
pub fn parent_of(rel: &str) -> Option<&str> {
    if rel.is_empty() {
        None
    } else {
        Some(rel.rsplit_once('/').map_or("", |(parent, _)| parent))
    }
}

/// The label of a folder's README: `README.md` at the root, and `<folder>/README.md` below it.
pub fn readme_label(dir: &str) -> String {
    if dir.is_empty() {
        "README.md".to_string()
    } else {
        format!("{dir}/README.md")
    }
}

/// Prints each manifest warning on its own stderr line. The loader calls this once per run, and a failed write is ignored,
/// as there is nowhere left to report it.
fn warn(lines: &[String]) {
    let mut stderr = io::stderr().lock();
    for line in lines {
        let _ = writeln!(stderr, "{line}");
    }
}

/// The last component of a root-relative path.
fn name_of(rel: &str) -> String {
    rel.rsplit('/').next().unwrap_or(rel).to_string()
}

/// The docs one walked file gives: its README, the cases of a CASES.md, or its header.
fn file_docs(entry: &Entry) -> Result<Vec<Doc>, Error> {
    match entry.path.file_name().and_then(|name| name.to_str()) {
        Some("README.md") => Ok(vec![readme_doc(entry)?]),
        Some("CASES.md") => Ok(cases::parse(&entry.rel, &read_text(&entry.path)?)),
        _ => Ok(header_doc(entry)?.into_iter().collect()),
    }
}

/// The doc of a README: its whole text, labelled by its path.
fn readme_doc(entry: &Entry) -> Result<Doc, Error> {
    let text = read_text(&entry.path)?;
    Ok(Doc {
        label: entry.rel.clone(),
        collection: "readmes".to_string(),
        id: entry.rel.clone(),
        path: Some(entry.rel.clone()),
        title: readme::one_liner(&text),
        text,
        fields: Vec::new(),
    })
}

/// The doc of a file's header, when the file has one.
fn header_doc(entry: &Entry) -> Result<Option<Doc>, Error> {
    let head = read_head(&entry.path)?;
    let Some(text) = header::header(&entry.path, &head) else {
        return Ok(None);
    };
    Ok(Some(Doc {
        label: entry.rel.clone(),
        collection: "headers".to_string(),
        id: entry.rel.clone(),
        path: Some(entry.rel.clone()),
        title: intent::lead(&text),
        text,
        fields: Vec::new(),
    }))
}

/// The first bytes of a file, at most `HEAD_BYTES`.
fn read_head(path: &Path) -> Result<Vec<u8>, Error> {
    let file = File::open(path).map_err(|failure| io_error(path, &failure))?;
    let mut head = Vec::new();
    file.take(HEAD_BYTES)
        .read_to_end(&mut head)
        .map_err(|failure| io_error(path, &failure))?;
    Ok(head)
}

/// The whole text of a file, read as the shared helper reads it (design §4.4 Text).
fn read_text(path: &Path) -> Result<String, Error> {
    text::read(path).map_err(|failure| io_error(path, &failure))
}

/// A read failure, naming the path and the io kind and never the OS text (design §4.4).
fn io_error(path: &Path, failure: &io::Error) -> Error {
    Error::io(&format!("cannot read {}", slash(path)), failure)
}
