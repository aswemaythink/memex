//! Carries out `memex show` (design §3.3 and §3.6).
//! It resolves each ref, builds its section, and reports the sections, the misses and the exit status.
//! Cases: SHOW-FILE-01, SHOW-FILE-02, SHOW-FILE-03, SHOW-MD-01, SHOW-MD-02, SHOW-DIR-01, SHOW-DIR-02, SHOW-ROOT-01, SHOW-ROOT-02, SHOW-ROOT-03, SHOW-LOG-01, SHOW-LOG-02, SHOW-UP-01, SHOW-UP-02, SHOW-LIMIT-01, SHOW-LIMIT-02, SHOW-SEP-01, SHOW-ERR-01, SHOW-ERR-02, SHOW-LINE-01, SHOW-JSON-01, SHOW-JSON-02, SHOW-JSON-03, SHOW-JSON-04, SHOW-JSON-05, CASE-SECT-01, CASE-JSON-01, REF-WT-01, REF-WT-02, CITE-PREFIX-01, CITE-PREFIX-02, CITE-UNRES-01, CITE-FOLDER-02, CITE-SELF-01, CITE-BACK-01, CITE-BACK-03, CITE-BACK-04, CITE-MAN-01, CITE-COLL-02, CITE-JSON-01, CITE-JSON-02, SHOW-ERR-03, SHOW-DUP-01, SHOW-DUP-02

use std::collections::BTreeSet;
use std::fs::File;
use std::io;
use std::path::Path;

use crate::cite::Cite;
use crate::cli::Verb;
use crate::corpus::{self, parent_of, readme_label, Child, Corpus, Doc, Kind};
use crate::error::Error;
use crate::intent::cases;
use crate::render::{self, CaseParts, Fields, Link, Listing, Miss, Ref, Section};
use crate::resolve::{resolve, Elsewhere, Outcome, Target};
use crate::scan::fail;
use crate::text;
use crate::verbs::here;

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

/// Shows each ref in the order given. A ref that fails is reported and the others still print. An empty ref is a usage error,
/// and a ref given twice prints once, at its first position (design §3.3). A requested file that cannot be read is that ref's
/// `io` error, so the verb exits 1 like any failed ref. A fatal problem, such as an unreadable cwd, is an error.
pub fn run(request: &Request) -> Result<Report, Error> {
    if request.refs.iter().any(String::is_empty) {
        return Err(fail(Verb::Show, "empty ref".to_string()));
    }
    let here::Here { cwd, store, corpus } = here::load()?;
    show_in(&corpus, &cwd, store.as_deref(), request)
}

/// Shows the refs of `request` as typed from `cwd`, in the corpus of their root. `run` passes the cwd's own corpus; a test can
/// pass one that it built.
fn show_in(
    corpus: &Corpus,
    cwd: &Path,
    store: Option<&Path>,
    request: &Request,
) -> Result<Report, Error> {
    let options = Options {
        up: request.up,
        limit: request.limit.unwrap_or(DEFAULT_LIMIT),
    };
    let mut sections = Vec::new();
    let mut misses = Vec::new();
    let mut seen = BTreeSet::new();
    for arg in &request.refs {
        if !seen.insert(arg.as_str()) {
            continue;
        }
        let built = match resolve(corpus, cwd, store, arg)? {
            Outcome::Found(target) => section_for(corpus, &target, &options),
            Outcome::Elsewhere(other) => elsewhere_section(&other, &options),
            Outcome::NotFound => {
                misses.push(Miss::not_found(arg));
                continue;
            }
            Outcome::Ambiguous(candidates) => {
                misses.push(Miss::ambiguous(arg, candidates));
                continue;
            }
        };
        match built {
            Ok(section) => sections.push(section),
            Err(failure) => misses.push(Miss::unreadable(arg, &failure)),
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

/// The section of an entry in another root: built from that root's own corpus, and labelled by its absolute path, which
/// round-trips from anywhere (design §3.1). Its cites and cited by name labels of that root, which do not round-trip from
/// the cwd, so they print as none until absolute refs are designed for them.
fn elsewhere_section(other: &Elsewhere, options: &Options) -> Result<Section, io::Error> {
    let target = Target::Path {
        rel: other.rel.clone(),
        kind: other.kind,
    };
    let mut section = section_for(&other.corpus, &target, options)?;
    section.label.clone_from(&other.label);
    section.cites.clear();
    section.cited_by = Listing {
        total: 0,
        items: Vec::new(),
    };
    Ok(section)
}

/// The section of one resolved target. A file that cannot be read gives its io error (design §3.3).
fn section_for(
    corpus: &Corpus,
    target: &Target<'_>,
    options: &Options,
) -> Result<Section, io::Error> {
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
        cites: readme
            .map(|doc| cites_of(corpus, &doc.label))
            .unwrap_or_default(),
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
/// starts its chain one folder higher, since its own text is the body. A file that cannot be read gives its io error.
fn file_section(corpus: &Corpus, rel: &str, options: &Options) -> Result<Section, io::Error> {
    let doc = corpus.doc(rel);
    let path = corpus.root.join(rel);
    let text = if rel.ends_with(".md") {
        Some(text::read(&path)?)
    } else {
        File::open(&path)?;
        doc.map(|doc| doc.text.clone())
    };
    let folder = parent_of(rel).unwrap_or("");
    let start = if is_readme(rel) {
        parent_of(folder)
    } else {
        Some(folder)
    };
    // A manifest doc's citations and backlinks are keyed by its label, not its path (design §2.3).
    let key = corpus.citations().manifest_label(rel).unwrap_or(rel);
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
        cites: doc
            .map(|doc| cites_of(corpus, &doc.label))
            .unwrap_or_else(|| cites_of(corpus, key)),
        cited_by: cited_by(corpus, key, options.limit),
        chain: chain(corpus, start, options),
    })
}

/// The section of a doc, such as a log or a case: its text, its fields, and its citations, with no chain.
fn doc_section(corpus: &Corpus, doc: &Doc, options: &Options) -> Section {
    Section {
        label: doc.label.clone(),
        title: Some(doc.title.clone()),
        collection: Some(doc.collection.clone()),
        id: Some(doc.id.clone()),
        path: doc.path.clone(),
        text: Some(doc.text.clone()),
        missing: "(no text)",
        fields: Some(doc_fields(corpus, doc)),
        children: None,
        // A case shows cited by and no cites (design §3.3 Case), so its JSON cites list is empty too.
        cites: if doc.collection == corpus::CASES {
            Vec::new()
        } else {
            cites_of(corpus, &doc.label)
        },
        cited_by: cited_by(corpus, &doc.label, options.limit),
        chain: None,
    }
}

/// The fields of a doc: a case's group and the evidence its text names that the walk holds (design §2.2), or the
/// frontmatter of any other doc.
fn doc_fields(corpus: &Corpus, doc: &Doc) -> Fields {
    if doc.collection == corpus::CASES {
        Fields::Case(CaseParts {
            group: cases::group_of(doc),
            evidence: cases::evidence(doc, |rel| corpus.kind(rel) == Some(Kind::File)),
        })
    } else {
        Fields::Pairs(doc.fields.clone())
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

/// The docs that cite `label`, each with its one-liner and the prefix it cites by, with the limit applied.
fn cited_by(corpus: &Corpus, label: &str, limit: usize) -> Listing<Ref> {
    let refs: Vec<Ref> = corpus
        .citations()
        .cited_by(label)
        .iter()
        .map(|citer| Ref {
            label: citer.label.clone(),
            title: Some(citer.title.clone()),
            via: citer.via.clone(),
        })
        .collect();
    Listing::of(&refs, limit)
}

/// The citations of the doc with this label, in the order the index holds them.
fn cites_of(corpus: &Corpus, label: &str) -> Vec<Cite> {
    corpus.citations().cites(label).to_vec()
}

/// True for a README.md, at the root or in a folder.
fn is_readme(rel: &str) -> bool {
    rel == "README.md" || rel.ends_with("/README.md")
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    /// A requested file that cannot be read fails its ref with an `io` error, in text and in JSON, and the other refs still print
    /// (design §3.3). Setup cannot chmod, so the test does.
    #[test]
    fn a_requested_file_that_cannot_be_read_is_its_refs_io_error() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let root = dir.path();
        fs::write(root.join("README.md"), "# Demo\n\nThe demo.\n").expect("a file");
        fs::write(root.join("notes.md"), "# Notes\n\nNotes.\n").expect("a file");
        fs::set_permissions(root.join("README.md"), fs::Permissions::from_mode(0o000))
            .expect("a mode");
        if fs::read(root.join("README.md")).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let corpus = Corpus::load(root, None).expect("the layer loads");
        let request = |json: bool| Request {
            refs: vec!["README.md".to_string(), "notes.md".to_string()],
            up: None,
            limit: None,
            json,
        };
        let text = show_in(&corpus, root, None, &request(false)).expect("show runs");
        let json = show_in(&corpus, root, None, &request(true)).expect("show runs");
        fs::set_permissions(root.join("README.md"), fs::Permissions::from_mode(0o644))
            .expect("restored");
        assert!(text.failed);
        assert_eq!(
            text.stderr,
            "memex: cannot read README.md (PermissionDenied)\n"
        );
        assert!(text.stdout.contains("=== notes.md ==="), "{}", text.stdout);
        assert!(json.stdout.contains(r#""code":"io""#), "{}", json.stdout);
    }
}
