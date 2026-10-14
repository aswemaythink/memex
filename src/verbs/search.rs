//! Carries out `memex search` (design §3.2 and §3.6): gathers, groups and ranks docs, then prints.
//! Cases: SRCH-RANK-01, SRCH-RANK-02, SRCH-WEIGHT-01, SRCH-WEIGHT-02, SRCH-STEM-01, SRCH-PREFIX-01, SRCH-PREFIX-02, SRCH-FUZZY-01, SRCH-FUZZY-02, SRCH-TIE-01, SRCH-TOKEN-01, SRCH-TOKEN-02, SRCH-GROUP-01, SRCH-GROUP-02, SRCH-LIMIT-01, SRCH-IN-01, SRCH-IN-02, SRCH-RANK-03, SRCH-TOKEN-03, SRCH-JSON-01, SRCH-JSON-02, SRCH-LINE-01

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::clock::{self, Timestamp};
use crate::collections::manifest;
use crate::corpus::{self, Corpus, Doc};
use crate::error::{Code, Error};
use crate::index::tokenize::query_tokens;
use crate::index::{Index, Source};
use crate::intent::cases;
use crate::logs::read as log_docs;
use crate::render::search::{self as render, Group, Hit};
use crate::root::{self, slash};
use crate::walk;

/// The `--limit` of each group when `--in` is not given (design §3.2).
const DEFAULT_LIMIT: usize = 5;

/// The `--limit` of each group when `--in` is given (design §3.2).
const IN_LIMIT: usize = 20;

/// The lines of a header that the index reads; `show` prints the whole header (design §2.1).
const HEADER_LINES: usize = 60;

/// The built-in groups of the intent layer, which are also collection names (design §3.1).
const READMES: &str = "readmes";
const HEADERS: &str = "headers";
const CASES: &str = "cases";

/// The group that holds every log stream (design §3.2).
const LOGS: &str = "logs";

/// The length of the stamp that starts a log's file name: `YYYYMMDD-HHMMSS` (design §4.3).
const STAMP_LEN: usize = 15;

/// The arguments of `memex search` once the grammar has passed them.
pub struct Request {
    /// The words and filters, in the order given. Only words are searched until S3.4.
    pub terms: Vec<String>,
    /// The groups named by `--in`, if the flag was given.
    pub in_: Option<Vec<String>>,
    /// The per-group `--limit`, if the flag was given.
    pub limit: Option<usize>,
    /// True when `--json` was given.
    pub json: bool,
}

/// What a search asks of the index: its words, the groups it reads, the limit of each group, and the instant that log ages are
/// measured from.
struct Plan<'a> {
    words: Vec<String>,
    groups: Option<&'a [String]>,
    limit: usize,
    now: Timestamp,
}

/// Carries out a search and returns its stdout: the text of its groups, or the JSON document with `--json`. Zero hits give no
/// text, and the exit status is 0 as usual.
pub fn run(request: &Request) -> Result<String, Error> {
    let cwd = current_dir()?;
    let store = store_of(&cwd)?;
    let root = corpus::root_of(&cwd);
    let corpus = Corpus::load(&root, store.as_deref())?;
    let plan = Plan {
        words: words_of(&request.terms),
        groups: request.in_.as_deref(),
        limit: request.limit.unwrap_or(if request.in_.is_some() {
            IN_LIMIT
        } else {
            DEFAULT_LIMIT
        }),
        now: clock::now()?,
    };
    let docs = gather(&corpus, &root, store.as_deref())?;
    let groups = rank_groups(docs, &plan);
    Ok(if request.json {
        render::document(&request.terms.join(" "), &groups)
    } else {
        render::text(&groups)
    })
}

/// Every doc of the intent layer and of the log store, each with its group. The readers of the docs give their labels, and each
/// doc is then taken from `corpus`, so the text that a search reads is the text that `show` prints (design §3.1).
fn gather<'a>(
    corpus: &'a Corpus,
    root: &Path,
    store: Option<&Path>,
) -> Result<Vec<(String, &'a Doc)>, Error> {
    let mut labels = BTreeSet::new();
    intent_labels(root, &mut labels)?;
    let mut scratch = Vec::new();
    let manifests = manifest::load(root, &mut scratch);
    for found in &manifests {
        let docs = manifest::docs(found, root, &mut scratch);
        labels.extend(docs.into_iter().map(|doc| doc.label));
    }
    if let Some(store) = store {
        labels.extend(log_docs::load(store)?.into_iter().map(|doc| doc.label));
    }
    let names: BTreeSet<&str> = manifests.iter().map(|found| found.name.as_str()).collect();
    Ok(labels
        .iter()
        .filter_map(|label| corpus.doc(label))
        .map(|doc| (group_of(doc, &names), doc))
        .collect())
}

/// The labels of the intent layer's docs: a file's own path, which labels its README or its header, and the case ids of each
/// CASES.md. A file with no doc adds a label that no doc has, which is skipped later.
fn intent_labels(root: &Path, labels: &mut BTreeSet<String>) -> Result<(), Error> {
    for entry in walk::walk(root).iter().filter(|entry| !entry.is_dir) {
        if entry.path.file_name() == Some(OsStr::new("CASES.md")) {
            let text = read_text(&entry.path)?;
            labels.extend(
                cases::parse(&entry.rel, &text)
                    .into_iter()
                    .map(|doc| doc.label),
            );
        } else {
            labels.insert(entry.rel.clone());
        }
    }
    Ok(())
}

/// The group of a doc: its collection when that is a built-in group or a manifest's name, and otherwise the log store, whose
/// streams are all one group.
fn group_of(doc: &Doc, manifests: &BTreeSet<&str>) -> String {
    let built_in = [READMES, HEADERS, CASES].contains(&doc.collection.as_str());
    if built_in || manifests.contains(doc.collection.as_str()) {
        doc.collection.clone()
    } else {
        LOGS.to_string()
    }
}

/// The groups that hold hits, in the order of design §3.2. A group that `--in` leaves out is not searched, and a group with no
/// hit is left out. With no words there are no hits.
fn rank_groups<'a>(docs: Vec<(String, &'a Doc)>, plan: &Plan) -> Vec<Group<'a>> {
    if plan.words.is_empty() {
        return Vec::new();
    }
    let mut groups: BTreeMap<(u8, String), Vec<&'a Doc>> = BTreeMap::new();
    for (name, doc) in docs {
        if wanted(plan, &name) {
            groups.entry((rank(&name), name)).or_default().push(doc);
        }
    }
    groups
        .into_iter()
        .filter_map(|((_, name), docs)| search_group(name, &docs, plan))
        .collect()
}

/// True when there is no `--in` list, or when the list names the group.
fn wanted(plan: &Plan, name: &str) -> bool {
    plan.groups
        .is_none_or(|names| names.iter().any(|named| named.as_str() == name))
}

/// The place of a group in the output: 0 for readmes, 1 for headers, 2 for cases, 3 for a manifest's collection and 4 for logs.
fn rank(name: &str) -> u8 {
    match name {
        READMES => 0,
        HEADERS => 1,
        CASES => 2,
        LOGS => 4,
        _ => 3,
    }
}

/// The group of docs that the words search: its total, and its hits within the limit. None when no doc matches.
fn search_group<'a>(name: String, docs: &[&'a Doc], plan: &Plan) -> Option<Group<'a>> {
    let sources: Vec<Source> = docs.iter().map(|doc| source_of(doc)).collect();
    let hits = Index::build(&sources).search(&plan.words);
    let total = hits.len();
    if total == 0 {
        return None;
    }
    let shown = hits
        .into_iter()
        .take(plan.limit)
        .map(|hit| {
            let doc = docs[hit.doc];
            Hit {
                doc,
                score: hit.score,
                fuzzy: hit.fuzzy,
                age: age_of(&name, doc, plan.now),
            }
        })
        .collect();
    Some(Group {
        name,
        total,
        hits: shown,
    })
}

/// A doc as the index reads it. A header is cut to its first `HEADER_LINES` lines; the text of every other doc is read whole.
fn source_of(doc: &Doc) -> Source<'_> {
    let body = if doc.collection == HEADERS {
        first_lines(&doc.text, HEADER_LINES)
    } else {
        &doc.text
    };
    Source {
        label: &doc.label,
        id: &doc.id,
        title: &doc.title,
        body,
    }
}

/// The text up to the end of its `count`th line, without that line's break. A text with fewer lines is whole.
fn first_lines(text: &str, count: usize) -> &str {
    if count == 0 {
        return "";
    }
    let end = text
        .match_indices('\n')
        .nth(count - 1)
        .map_or(text.len(), |(at, _)| at);
    &text[..end]
}

/// The age of a log entry, from the stamp that starts its file name, which is the instant of its `time` field. Other docs have no
/// age, and nor does a log whose name has no stamp.
fn age_of(group: &str, doc: &Doc, now: Timestamp) -> Option<String> {
    if group != LOGS {
        return None;
    }
    doc.id
        .get(..STAMP_LEN)
        .and_then(clock::parse_stamp)
        .map(|stamp| render::age(now.seconds_since(stamp)))
}

/// The query's words: every token of every term in order, with no repeats. A term that holds no token adds none.
fn words_of(terms: &[String]) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for token in terms.iter().flat_map(|term| query_tokens(term)) {
        if !words.contains(&token) {
            words.push(token);
        }
    }
    words
}

/// The cwd, canonical, so that it compares with the canonical root.
fn current_dir() -> Result<PathBuf, Error> {
    let cwd = env::current_dir()
        .map_err(|failure| Error::io("cannot read the current folder", &failure))?;
    dunce::canonicalize(&cwd)
        .map_err(|failure| Error::io("cannot resolve the current folder", &failure))
}

/// The log store, or None when there is no `.memex/`. A search reads the logs when there is a store, and needs none.
fn store_of(cwd: &Path) -> Result<Option<PathBuf>, Error> {
    match root::locate(cwd) {
        Ok(place) => Ok(Some(place.store)),
        Err(error) if error.code == Code::NoRoot => Ok(None),
        Err(error) => Err(error),
    }
}

/// The text of a file, decoded lossily. A read failure names the path and never the OS text (design §4.4).
fn read_text(path: &Path) -> Result<String, Error> {
    let bytes = fs::read(path)
        .map_err(|failure| Error::io(&format!("cannot read {}", slash(path)), &failure))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
