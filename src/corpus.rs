//! Loads the read model (design §3.1 and §3.6): the walked entries and the docs built from them.
//! The docs are READMEs, headers, collection docs read from frontmatter and manifests, and logs, with the label map.
//! The walked entries and the intent docs are bulk-built from sorted input; manifest and log docs are inserted one by one.
//! Cases: SHOW-FILE-01, SHOW-FILE-02, SHOW-DIR-01, SHOW-DIR-02, SHOW-ORDER-01, SHOW-ROOT-01, SHOW-ROOT-02, SHOW-LOG-01, SHOW-LOG-02, SHOW-LINE-01, SHOW-IO-02, INT-NAME-01, INT-NAME-02, INT-CHAIN-01, INT-BOM-01, MAN-LOAD-02, REF-AMBIG-02, REF-AMBIG-03, REF-RESERVED-03, REF-RESERVED-04, COLL-INFO-01, COLL-INFO-02, COLL-SHARED-01, COLL-SHARED-02, COLL-SHARED-03, COLL-SHARED-04, COLL-DUP-01, COLL-DUP-02, COLL-DUP-03, SHOW-MDCHILD-01, SHOW-MDCHILD-02, SHOW-MDCHILD-03, SHOW-MDCHILD-04

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::ops::Bound;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread;

use crate::cite;
use crate::collections::frontmatter::Value;
use crate::collections::manifest;
use crate::error::{self, Error};
use crate::intent::{by_label, files, front, shared, Origin};
use crate::logs::read::load as load_logs;
use crate::par;
use crate::walk;

pub use crate::root::root_of;

/// One indexed doc: an intent doc labelled by its path, or a collection doc labelled `collection:id` (design §3.1 and §3.6).
#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    /// The label `show` takes: a root-relative path for an intent doc, `collection:id` for any other.
    pub label: String,
    /// The collection: `readmes` or `headers` for the graph, a collection's name, or a log's stream.
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
    /// Where the doc was read from (design §2.2 and §4.2).
    pub origin: Origin,
}

/// The built-in collection of READMEs, whose docs are labelled by their path (design §3.1).
pub const READMES: &str = "readmes";
/// The built-in collection of headers, whose docs are labelled by their path (design §3.1).
pub const HEADERS: &str = "headers";
/// The built-in name of the log store: `logs:` names an entry of any stream (design §3.1).
pub const LOGS: &str = "logs";
/// The names of the graph and the log store (design §3.1): a manifest and `log` may not take one, and a `collection:id` label may
/// always use one. A frontmatter collection may take one, with a warning (design §4.2 Names).
pub const RESERVED: [&str; 3] = [READMES, HEADERS, LOGS];

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
    /// The one-liner of a folder's README, or of a file's header, README or Markdown text; None when there is none.
    pub title: Option<String>,
}

/// The read model: every doc by label, every walked entry, and the log streams. Folders list their children on request.
pub struct Corpus {
    /// The root the intent layer is read from.
    pub root: PathBuf,
    /// Every doc by label, in load order. A label holds more than one doc when two files declare the same id.
    docs: BTreeMap<String, Vec<Doc>>,
    kinds: BTreeMap<String, Kind>,
    /// The title of each Markdown file that was read, by root-relative path: what its folder's child line shows (design §3.3).
    titles: BTreeMap<String, String>,
    /// The folders that the walk could not list, by root-relative path, with the kind of the failure (design §3.3).
    unlistable: BTreeMap<String, io::ErrorKind>,
    /// The streams of the log store, which `logs:` reads across (design §3.1).
    streams: BTreeSet<String>,
    /// The citations of every doc and the backlinks they give, built the first time they are asked for (design §2.3).
    citations: OnceLock<cite::Index>,
    /// The warnings of the files, folders and log entries that could not be read as the corpus loaded (design §4.2). `info` lists
    /// them in its JSON.
    pub read_warnings: Vec<String>,
    /// The manifests that loaded, in file-name order, then the collections that only frontmatter names, each with its doc count and
    /// the warnings of its read (design §2.2 and §4.2).
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
    /// their counts and warnings for `info` and `search`. The files are read on several threads. The walk's entries are sorted, so
    /// the kinds are bulk-built from them, and the docs are grouped by label in one pass that keeps the walk's order within each
    /// label. A file that cannot be read is skipped with its warning, and a folder that cannot be listed is kept as unlistable
    /// (design §3.3). The rest of the layer loads (design §4.2).
    fn intent(root: &Path) -> Corpus {
        let (mut read_warnings, mut unlistable) = (Vec::new(), BTreeMap::new());
        let entries = walk::walk_warned(root, &mut read_warnings, &mut unlistable);
        let read = par::map(&entries, files::read_entry);
        let mut kinds = Vec::with_capacity(entries.len() + 1);
        kinds.push((String::new(), Kind::Dir));
        let mut docs = Vec::new();
        let mut titles = BTreeMap::new();
        for (entry, found) in entries.into_iter().zip(read) {
            let kind = if entry.is_dir { Kind::Dir } else { Kind::File };
            match found {
                Ok(found) => {
                    titles.extend(found.title.map(|title| (entry.rel.clone(), title)));
                    docs.extend(found.docs);
                    read_warnings.extend(found.warnings.iter().map(|line| error::warning(line)));
                }
                Err(error) => read_warnings.push(error::warning(&error.message)),
            }
            kinds.push((entry.rel, kind));
        }
        let mut corpus = Corpus {
            root: root.to_path_buf(),
            docs: by_label(docs),
            kinds: kinds.into_iter().collect(),
            titles,
            unlistable,
            streams: BTreeSet::new(),
            citations: OnceLock::new(),
            read_warnings,
            manifests: Vec::new(),
            manifest_warnings: Vec::new(),
        };
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
        let counts = front::counts(corpus.all_docs());
        front::join(counts, &mut corpus.manifests);
        error::warn(&corpus.read_warnings);
        error::warn(&corpus.manifest_warnings);
        for loaded in &corpus.manifests {
            error::warn(&loaded.warnings);
        }
        corpus
    }

    /// Adds the logs of the store, when there is one, then the shared-name warnings, which need the streams (design §4.2 Names).
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
        let shared = shared::name_warnings(self);
        error::warn(&shared);
        self.read_warnings.extend(shared);
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

    /// True for a log entry, by its origin (design §4.3): a collection named like a stream is never one.
    pub fn is_log(&self, doc: &Doc) -> bool {
        doc.origin == Origin::Log
    }

    /// True when the log store holds an entry of the stream `name` (design §3.4).
    pub fn is_stream(&self, name: &str) -> bool {
        self.streams.contains(name)
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

    /// The kind of the io error that kept the walk from listing this folder, or None when the walk listed it or never found it
    /// (design §3.3). A request for such a folder is that ref's io error.
    pub fn unlistable(&self, rel: &str) -> Option<io::ErrorKind> {
        self.unlistable.get(rel).copied()
    }

    /// The children of a folder, sorted by name: the walked entries directly under it. Empty for a file and for a path the walk did
    /// not find. The entries under a folder are one range of `kinds`, and only its children get a title.
    pub fn children(&self, rel: &str) -> Vec<Child> {
        let prefix = if rel.is_empty() {
            String::new()
        } else {
            format!("{rel}/")
        };
        self.kinds
            .range::<str, _>((Bound::Included(prefix.as_str()), Bound::Unbounded))
            .take_while(|(path, _)| path.starts_with(&prefix))
            .filter(|(path, _)| direct_child_of(path, &prefix))
            .map(|(path, kind)| Child {
                name: name_of(path).to_string(),
                rel: path.clone(),
                kind: *kind,
                title: self.title_of(path, *kind),
            })
            .collect()
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

    /// The one-liner of a walked entry: its README's for a folder, its own doc's for a file, else a Markdown file's title.
    fn title_of(&self, rel: &str, kind: Kind) -> Option<String> {
        let doc = match kind {
            Kind::Dir => self.doc(&readme_label(rel)),
            Kind::File => self.doc(rel),
        };
        doc.map(|doc| doc.title.clone())
            .or_else(|| self.titles.get(rel).cloned())
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

/// True for a Markdown file by its name: the walk reads a name that ends in `.md` as Markdown (design §2.2).
pub fn is_markdown(name: &str) -> bool {
    name.ends_with(".md")
}

/// True for a root-relative path that is directly under the folder `prefix`: `""` for the root, else `<folder>/`. The root's own
/// empty path is no child, and a path with a `/` after the prefix lies deeper.
fn direct_child_of(path: &str, prefix: &str) -> bool {
    path.strip_prefix(prefix)
        .is_some_and(|name| !name.is_empty() && !name.contains('/'))
}
