//! Loads the read model (design §3.1 and §3.6): the walked entries and the docs built from them.
//! The docs are READMEs, headers, CASES.md, manifests and logs, with the label map and the folder index.
//! Cases: SHOW-FILE-01, SHOW-FILE-02, SHOW-DIR-01, SHOW-DIR-02, SHOW-ROOT-01, SHOW-ROOT-02, SHOW-LOG-01, SHOW-LOG-02, SHOW-LINE-01, INT-NAME-01, INT-NAME-02, INT-CHAIN-01, INT-BOM-01, MAN-LOAD-02, REF-AMBIG-02, REF-AMBIG-03, REF-RESERVED-03, REF-RESERVED-04

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread;

use crate::cite;
use crate::collections::frontmatter::Value;
use crate::collections::manifest;
use crate::error::{self, Error};
use crate::intent::files::file_docs;
use crate::logs::read::load as load_logs;
use crate::par;
use crate::walk::{self, Entry};

pub use crate::root::root_of;

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

/// The built-in collection of READMEs, whose docs are labelled by their path (design §3.1).
pub const READMES: &str = "readmes";
/// The built-in collection of headers, whose docs are labelled by their path (design §3.1).
pub const HEADERS: &str = "headers";
/// The built-in collection of case docs, which are labelled `cases:ID` (design §3.1).
pub const CASES: &str = "cases";
/// The built-in name of the log store: `logs:` names an entry of any stream (design §3.1).
pub const LOGS: &str = "logs";
/// The built-in collection names (design §3.1), which no manifest or stream may take. A `collection:id` label may always use
/// them, whether or not a doc holds them.
pub const RESERVED: [&str; 4] = [READMES, HEADERS, CASES, LOGS];

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
    /// The children of each folder, built the first time they are asked for.
    children: OnceLock<BTreeMap<String, Vec<Child>>>,
    /// The streams of the log store, which `logs:` reads across (design §3.1).
    streams: BTreeSet<String>,
    /// The citations of every doc and the backlinks they give, built the first time they are asked for (design §2.3).
    citations: OnceLock<cite::Index>,
    /// The warnings of the files, folders and log entries that could not be read as the corpus loaded (design §4.2). `info` lists
    /// them in its JSON.
    pub read_warnings: Vec<String>,
    /// The manifests that loaded, in file-name order, each with its doc count and the warnings of its read (design §4.2).
    pub manifests: Vec<manifest::Loaded>,
    /// The warnings of the manifest files that did not load, in file-name order.
    pub manifest_warnings: Vec<String>,
}

impl Corpus {
    /// Reads the intent layer under `root` and, when there is a store, the logs in it. A missing store has no logs.
    pub fn load(root: &Path, store: Option<&Path>) -> Corpus {
        let mut corpus = Corpus::intent(root);
        corpus.finish(store);
        corpus
    }

    /// Like `load`, with the store found by `find_store`, which runs on its own thread while the intent layer is read, since
    /// finding the store may run git. Returns the store too, or the error in finding it.
    pub fn load_finding<F>(root: &Path, find_store: F) -> Result<(Corpus, Option<PathBuf>), Error>
    where
        F: FnOnce() -> Result<Option<PathBuf>, Error> + Send,
    {
        let (mut corpus, store) = thread::scope(|scope| {
            let finder = scope.spawn(find_store);
            let intent = Corpus::intent(root);
            let store = finder
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
            (intent, store)
        });
        let store = store?;
        corpus.finish(store.as_deref());
        Ok((corpus, store))
    }

    /// The intent layer under `root`: every walked entry, the docs of its files, and the docs of its manifests, which are kept with
    /// their counts and warnings for `info` and `search`. The files are read on several threads, and their docs are added in the
    /// walk's order. A file that cannot be read is skipped with its warning, and
    /// the rest of the layer loads (design §4.2).
    fn intent(root: &Path) -> Corpus {
        let mut corpus = Corpus {
            root: root.to_path_buf(),
            docs: BTreeMap::new(),
            kinds: BTreeMap::from([(String::new(), Kind::Dir)]),
            children: OnceLock::new(),
            streams: BTreeSet::new(),
            citations: OnceLock::new(),
            read_warnings: Vec::new(),
            manifests: Vec::new(),
            manifest_warnings: Vec::new(),
        };
        let entries = walk::walk_warned(root, &mut corpus.read_warnings);
        let read = par::map(&entries, read_entry);
        for (entry, docs) in entries.iter().zip(read) {
            let kind = if entry.is_dir { Kind::Dir } else { Kind::File };
            corpus.kinds.insert(entry.rel.clone(), kind);
            match docs {
                Ok(docs) => docs.into_iter().for_each(|doc| corpus.insert(doc)),
                Err(error) => corpus.read_warnings.push(error::warning(&error.message)),
            }
        }
        for found in manifest::load(root, &mut corpus.manifest_warnings) {
            let mut warnings = Vec::new();
            let docs = manifest::docs(&found, root, &mut warnings);
            corpus.manifests.push(manifest::Loaded {
                manifest: found,
                count: docs.len(),
                warnings,
            });
            docs.into_iter().for_each(|doc| corpus.insert(doc));
        }
        error::warn(&corpus.read_warnings);
        error::warn(&corpus.manifest_warnings);
        for loaded in &corpus.manifests {
            error::warn(&loaded.warnings);
        }
        corpus
    }

    /// Adds the logs of the store, when there is one. The folder index and the citations wait until they are asked for.
    fn finish(&mut self, store: Option<&Path>) {
        if let Some(store) = store {
            let before = self.read_warnings.len();
            let logs = load_logs(store, &mut self.read_warnings);
            error::warn(&self.read_warnings[before..]);
            self.streams = logs.iter().map(|doc| doc.collection.clone()).collect();
            for doc in logs {
                self.insert(doc);
            }
        }
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

    /// The docs that a label names, as `show` reads one (design §3.1): the docs that hold the label, or else the docs that a
    /// reserved spelling names. `readmes:` and `headers:` name a doc by its path, and `logs:` names the entry of every stream
    /// that holds the id, so one id can name several entries.
    pub fn named(&self, label: &str) -> Vec<&Doc> {
        let held = self.labelled(label);
        if !held.is_empty() {
            return held.iter().collect();
        }
        let Some((collection, id)) = label.split_once(':') else {
            return Vec::new();
        };
        self.all_docs()
            .filter(|doc| match collection {
                LOGS => self.is_log(doc) && doc.id == id,
                READMES | HEADERS => doc.collection == collection && doc.id == id,
                _ => false,
            })
            .collect()
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

    /// The children of a folder, sorted by name. Empty for a file and for a path the walk did not find. The folder index is built
    /// the first time it is asked for, so a verb that does not list folders never pays for it.
    pub fn children(&self, rel: &str) -> &[Child] {
        match self.children.get_or_init(|| self.index_children()).get(rel) {
            Some(list) => list,
            None => &[],
        }
    }

    /// The citations of every doc and the backlinks they give (design §2.3). They are built on the first call, so a verb that does
    /// not cite, such as `info` or a words search, never pays for them.
    pub fn citations(&self) -> &cite::Index {
        self.citations.get_or_init(|| cite::Index::build(self))
    }

    /// Adds a doc under its label. Every doc is kept, so a label that two files declare is seen as ambiguous, not guessed.
    fn insert(&mut self, doc: Doc) {
        self.docs.entry(doc.label.clone()).or_default().push(doc);
    }

    /// Builds the folder index: every walked entry becomes a child of its parent folder.
    fn index_children(&self) -> BTreeMap<String, Vec<Child>> {
        let mut index: BTreeMap<String, Vec<Child>> = BTreeMap::new();
        for (rel, kind) in &self.kinds {
            let Some(parent) = parent_of(rel) else {
                continue;
            };
            let child = Child {
                name: name_of(rel).to_string(),
                rel: rel.clone(),
                kind: *kind,
                title: self.title_of(rel, *kind),
            };
            index.entry(parent.to_string()).or_default().push(child);
        }
        index
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

/// The last component of a root-relative path.
pub fn name_of(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

/// The docs of one walked entry: none for a folder, and those of its file otherwise (design §2 and §4.2).
fn read_entry(entry: &Entry) -> Result<Vec<Doc>, Error> {
    if entry.is_dir {
        return Ok(Vec::new());
    }
    file_docs(entry)
}
