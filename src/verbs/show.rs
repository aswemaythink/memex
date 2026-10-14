//! Carries out `memex show` (design §3.3 and §3.6): resolves each ref, builds its section, and reports the sections, the misses and the exit status.
//! Cases: SHOW-FILE-01, SHOW-FILE-02, SHOW-FILE-03, SHOW-MD-01, SHOW-MD-02, SHOW-DIR-01, SHOW-DIR-02, SHOW-ROOT-01, SHOW-ROOT-02, SHOW-ROOT-03, SHOW-LOG-01, SHOW-LOG-02, SHOW-UP-01, SHOW-UP-02, SHOW-LIMIT-01, SHOW-LIMIT-02, SHOW-SEP-01, SHOW-ERR-01, SHOW-ERR-02, SHOW-LINE-01, SHOW-JSON-01, SHOW-JSON-02, SHOW-JSON-03, SHOW-JSON-04, SHOW-JSON-05

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cite;
use crate::corpus::{self, parent_of, readme_label, Child, Corpus, Doc, Kind};
use crate::error::{Code, Error};
use crate::render::{self, Link, Listing, Miss, Ref, Section};
use crate::resolve::{resolve, Outcome, Target};
use crate::root::{self, slash};
use crate::walk::rel_string;

/// The `--limit` of a list when the flag is not given (design §3.3).
const DEFAULT_LIMIT: usize = 50;

/// The arguments of `memex show` once the grammar has passed them.
pub struct Request {
    /// The refs, in the order given.
    pub refs: Vec<String>,
    /// The value of `--up`, if the flag was given.
    pub up: Option<usize>,
    /// The value of `--limit`, if the flag was given.
    pub limit: Option<usize>,
    /// True when `--json` was given.
    pub json: bool,
}

/// What `show` prints: stdout, stderr, and whether a ref failed, which makes the exit status 1.
pub struct Report {
    /// The sections as text, or the JSON document with `--json`.
    pub stdout: String,
    /// One line per miss in text mode. Empty with `--json`, where the misses are in the document.
    pub stderr: String,
    /// True when at least one ref was not shown.
    pub failed: bool,
}

/// The settings that every section is built with.
struct Options {
    /// The most READMEs in a chain, or None for the whole chain.
    up: Option<usize>,
    /// The most rows of a children list or a cited-by list.
    limit: usize,
}

/// Shows each ref in the order given. A ref that fails is reported and the others still print. A fatal problem, such as an
/// unreadable cwd or a file that cannot be read, is an error.
pub fn run(request: &Request) -> Result<Report, Error> {
    let cwd = current_dir()?;
    let store = store_of(&cwd)?;
    let root = corpus::root_of(&cwd);
    let corpus = Corpus::load(&root, store.as_deref())?;
    let cwd_rel = cwd
        .strip_prefix(&corpus.root)
        .ok()
        .and_then(rel_string)
        .unwrap_or_default();
    let options = Options {
        up: request.up,
        limit: request.limit.unwrap_or(DEFAULT_LIMIT),
    };
    let mut sections = Vec::new();
    let mut misses = Vec::new();
    for arg in &request.refs {
        match resolve(&corpus, &cwd_rel, arg) {
            Outcome::Found(target) => sections.push(section_for(&corpus, &target, &options)?),
            Outcome::NotFound => misses.push(Miss::not_found(arg)),
            Outcome::Ambiguous(candidates) => misses.push(Miss::ambiguous(arg, candidates)),
        }
    }
    let failed = !misses.is_empty();
    if request.json {
        let stdout = render::json::document(&sections, &misses);
        return Ok(Report {
            stdout,
            stderr: String::new(),
            failed,
        });
    }
    Ok(Report {
        stdout: render::text::sections(&sections),
        stderr: render::text::misses(&misses),
        failed,
    })
}

/// The cwd, canonical, so that it compares with the canonical root.
fn current_dir() -> Result<PathBuf, Error> {
    let cwd = env::current_dir()
        .map_err(|failure| Error::io("cannot read the current folder", &failure))?;
    dunce::canonicalize(&cwd)
        .map_err(|failure| Error::io("cannot resolve the current folder", &failure))
}

/// The log store that `log` would write to, or None when there is no `.memex/` (design §4.1). Reads work without one.
fn store_of(cwd: &Path) -> Result<Option<PathBuf>, Error> {
    match root::locate(cwd) {
        Ok(place) => Ok(Some(place.store)),
        Err(error) if error.code == Code::NoRoot => Ok(None),
        Err(error) => Err(error),
    }
}

/// The section of one resolved target.
fn section_for(corpus: &Corpus, target: &Target<'_>, options: &Options) -> Result<Section, Error> {
    match target {
        Target::Path {
            rel,
            kind: Kind::Dir,
        } => Ok(folder_section(corpus, rel, options)),
        Target::Path {
            rel,
            kind: Kind::File,
        } => file_section(corpus, rel, options),
        Target::Doc(doc) => Ok(doc_section(corpus, doc, options)),
    }
}

/// The section of a folder: its README or `(no README)`, its children other than that README, its citations, and the chain
/// above it.
fn folder_section(corpus: &Corpus, rel: &str, options: &Options) -> Section {
    let label = corpus::label_of(rel);
    let readme = corpus.doc(&readme_label(rel));
    let children = listed_children(corpus, rel);
    Section {
        label: label.clone(),
        title: readme.map(|doc| doc.title.clone()),
        collection: None,
        id: None,
        path: Some(label.clone()),
        text: readme.map(|doc| doc.text.clone()),
        missing: "(no README)",
        fields: None,
        children: Some(Listing::of(&children, options.limit)),
        cites: readme.map(cite::cites).unwrap_or_default(),
        cited_by: cited_by(corpus, &label, options.limit),
        chain: chain(corpus, parent_of(rel), options),
    }
}

/// The children that a folder's section lists: all of them but its own README.md, which the body has already printed
/// (design §3.3).
fn listed_children(corpus: &Corpus, rel: &str) -> Vec<Child> {
    let own = readme_label(rel);
    corpus
        .children(rel)
        .iter()
        .filter(|child| !(child.kind == Kind::File && child.rel == own))
        .cloned()
        .collect()
}

/// The section of a file: its header, or its whole text for a Markdown file, and the chain above its folder. A README
/// starts its chain one folder higher, since its own text is the body.
fn file_section(corpus: &Corpus, rel: &str, options: &Options) -> Result<Section, Error> {
    let doc = corpus.doc(rel);
    let text = if rel.ends_with(".md") {
        Some(read_file(&corpus.root.join(rel))?)
    } else {
        doc.map(|doc| doc.text.clone())
    };
    let folder = parent_of(rel).unwrap_or("");
    let start = if is_readme(rel) {
        parent_of(folder)
    } else {
        Some(folder)
    };
    Ok(Section {
        label: rel.to_string(),
        title: doc.map(|doc| doc.title.clone()),
        collection: doc.map(|doc| doc.collection.clone()),
        id: Some(rel.to_string()),
        path: Some(rel.to_string()),
        text,
        missing: "(no header)",
        fields: None,
        children: None,
        cites: doc.map(cite::cites).unwrap_or_default(),
        cited_by: cited_by(corpus, rel, options.limit),
        chain: chain(corpus, start, options),
    })
}

/// The section of a doc, such as a log: its text, its fields and its citations, with no chain.
fn doc_section(corpus: &Corpus, doc: &Doc, options: &Options) -> Section {
    Section {
        label: doc.label.clone(),
        title: Some(doc.title.clone()),
        collection: Some(doc.collection.clone()),
        id: Some(doc.id.clone()),
        path: doc.path.clone(),
        text: Some(doc.text.clone()),
        missing: "(no text)",
        fields: Some(doc.fields.clone()),
        children: None,
        cites: cite::cites(doc),
        cited_by: cited_by(corpus, &doc.label, options.limit),
        chain: None,
    }
}

/// The READMEs from `start` up to the root, nearest first, at most `--up` of them. None when `--up 0` prints no chain.
fn chain(corpus: &Corpus, start: Option<&str>, options: &Options) -> Option<Vec<Link>> {
    if options.up == Some(0) {
        return None;
    }
    let mut links: Vec<Link> = Vec::new();
    let mut folder = start;
    while let Some(dir) = folder {
        if options.up.is_some_and(|up| links.len() >= up) {
            break;
        }
        if let Some(doc) = corpus.doc(&readme_label(dir)) {
            links.push(Link {
                path: doc.label.clone(),
                text: doc.text.clone(),
            });
        }
        folder = parent_of(dir);
    }
    Some(links)
}

/// The refs that cite `label`, each with its one-liner when it resolves, with the limit applied.
fn cited_by(corpus: &Corpus, label: &str, limit: usize) -> Listing<Ref> {
    let refs: Vec<Ref> = cite::cited_by(label)
        .into_iter()
        .map(|cited| Ref {
            title: corpus.doc(&cited).map(|doc| doc.title.clone()),
            label: cited,
        })
        .collect();
    Listing::of(&refs, limit)
}

/// The whole text of a file, decoded lossily. A read failure is an io error (exit 3).
fn read_file(path: &Path) -> Result<String, Error> {
    let bytes = fs::read(path)
        .map_err(|failure| Error::io(&format!("cannot read {}", slash(path)), &failure))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// True for a README.md, at the root or in a folder.
fn is_readme(rel: &str) -> bool {
    rel == "README.md" || rel.ends_with("/README.md")
}
