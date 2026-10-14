//! Carries out `memex info` (design §3.5 and §3.6): the atlas of where things are, as text or JSON.
//! Cases: INFO-HEAD-01, INFO-HEAD-02, INFO-INTENT-01, INFO-INTENT-02, INFO-INTENT-03, INFO-COLL-01, INFO-COLL-02, INFO-STREAM-01, INFO-STREAM-02, INFO-STREAM-03, INFO-STREAM-04, INFO-TO-01, INFO-TO-02, INFO-TO-03, INFO-TO-04, INFO-TO-05, INFO-HERE-01, INFO-HERE-02, INFO-HERE-03, INFO-HERE-04, INFO-USE-01, INFO-CAP-01, INFO-WARN-01, INFO-WARN-02, INFO-NOROOT-01, INFO-NOROOT-02, INFO-JSON-01, INFO-JSON-02, INFO-JSON-03, CASE-DUP-02, INIT-INFO-03, INFO-ZERO-01, INFO-ZERO-02

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::cli::Verb;
use crate::clock::{self, Timestamp};
use crate::collections::frontmatter::Value;
use crate::collections::manifest::{Kind, Loaded};
use crate::corpus::{label_of, parent_of, readme_label, Corpus, Doc, CASES, HEADERS, READMES};
use crate::error::{self, Error};
use crate::logs::values;
use crate::render::info::{self as render, Atlas, Collection, HereLine, Intent, Stream, ToCount};
use crate::root::slash;
use crate::scan::fail;
use crate::verbs::here::{self, Here};

/// A stream entry is new when it is no more than this many seconds old: 24 hours (design §3.5 `new_24h`).
const NEW_SECONDS: i64 = 24 * 3_600;

/// The arguments of `memex info` once the grammar has passed them.
pub struct Request {
    /// The `--agent` id, if the flag was given.
    pub agent: Option<String>,
    /// True when `--json` was given.
    pub json: bool,
}

/// Carries out `info` and returns its stdout: the text of the atlas, or its JSON document with `--json`. Each duplicate case id
/// is warned on stderr here; the manifests that do not load were warned when the corpus loaded.
pub fn run(request: &Request, now: Timestamp) -> Result<String, Error> {
    let agent = agent_of(request.agent.as_deref())?;
    Ok(render_here(&here::load()?, agent, now, request.json))
}

/// The text atlas of `dir`, a canonical folder, as `init` ends with it (design §3.5b item 4). init has no `--agent` and does not
/// read `$MEMEX_AGENT`, so its atlas has no `to` line and an invalid value cannot fail it after its files are written.
/// Case: INIT-INFO-03
pub fn text_in(dir: PathBuf, now: Timestamp) -> Result<String, Error> {
    Ok(render_here(&here::load_in(dir)?, None, now, false))
}

/// The atlas of `here` for `agent`, as text or as its JSON document. Each duplicate case id is warned on stderr.
fn render_here(here: &Here, agent: Option<String>, now: Timestamp, json: bool) -> String {
    let duplicates = duplicate_warnings(&here.corpus);
    error::warn(&duplicates);
    let atlas = atlas(here, agent, now, &duplicates);
    if json {
        render::document(&atlas)
    } else {
        render::text(&atlas)
    }
}

/// The agent id from `--agent`, else `$MEMEX_AGENT`, checked as `log` checks it (design §3.4). Its usage error names `info`.
fn agent_of(flag: Option<&str>) -> Result<Option<String>, Error> {
    values::agent(flag, values::env_agent().as_deref())
        .map_err(|error| fail(Verb::Info, error.message))
}

/// The atlas of a run: the root and the store, the counts, the collections and streams, the `to` count, the `here` line, and the
/// warnings of the manifests, the duplicate case ids and the files that could not be read.
fn atlas(here: &Here, agent: Option<String>, now: Timestamp, duplicates: &[String]) -> Atlas {
    let corpus = &here.corpus;
    let (collections, mut warnings) = manifest_collections(corpus);
    warnings.extend(duplicates.iter().cloned());
    warnings.extend(corpus.read_warnings.iter().cloned());
    let store = here.store.as_deref();
    Atlas {
        root: slash(&corpus.root),
        store: store.map(slash),
        store_label: store_label(&corpus.root, store),
        intent: intent_of(corpus),
        collections,
        streams: streams_of(corpus, now),
        to: agent.map(|agent| to_of(corpus, agent)),
        here: here_of(corpus, &here.cwd),
        warnings,
    }
}

/// The store as the header prints it (design §3.5): root-relative with a trailing `/` when it is inside the root, else absolute;
/// `none (run memex init)` when there is no store.
fn store_label(root: &Path, store: Option<&Path>) -> String {
    match store {
        None => "none (run memex init)".to_string(),
        Some(store) => match store.strip_prefix(root) {
            Ok(rel) if !rel.as_os_str().is_empty() => format!("{}/", slash(rel)),
            _ => slash(store),
        },
    }
}

/// The manifests that loaded, sorted by name, each with its doc count (design §3.5 Collections), and the warnings of the manifest
/// reads: the files that did not load, then each collection's in name order. The corpus read them once and keeps them.
fn manifest_collections(corpus: &Corpus) -> (Vec<Collection>, Vec<String>) {
    let mut loaded: Vec<&Loaded> = corpus.manifests.iter().collect();
    loaded.sort_by(|left, right| left.manifest.name.cmp(&right.manifest.name));
    let mut warnings = corpus.manifest_warnings.clone();
    let collections = loaded
        .into_iter()
        .map(|found| {
            warnings.extend(found.warnings.iter().cloned());
            collection_of(found)
        })
        .collect();
    (collections, warnings)
}

/// One loaded manifest as a collection: its name, kind, description and owner, and the number of docs its reader gave.
fn collection_of(found: &Loaded) -> Collection {
    let manifest = &found.manifest;
    Collection {
        name: manifest.name.clone(),
        kind: kind_word(manifest.kind),
        count: found.count,
        description: manifest.description.clone(),
        owner: manifest.owner.clone(),
    }
}

/// The word that a manifest's `kind` key takes (design §4.2).
fn kind_word(kind: Kind) -> &'static str {
    match kind {
        Kind::Markdown => "markdown",
        Kind::Folder => "folder",
        Kind::Json => "json",
    }
}

/// The built-in docs of the intent layer, counted by collection (design §3.5 Header). A log entry never counts, even when its
/// stream carries a built-in name.
fn intent_of(corpus: &Corpus) -> Intent {
    let mut intent = Intent::default();
    for doc in corpus.all_docs().filter(|doc| !corpus.is_log(doc)) {
        match doc.collection.as_str() {
            READMES => intent.readmes += 1,
            HEADERS => intent.headers += 1,
            CASES => intent.cases += 1,
            _ => {}
        }
    }
    intent
}

/// The log streams, newest entry first and then by name (design §3.5 Streams): each with its entry count, its new entries and the
/// stamp of its newest entry. A stream with no stamped entry comes last.
fn streams_of(corpus: &Corpus, now: Timestamp) -> Vec<Stream> {
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for doc in corpus.all_docs().filter(|doc| corpus.is_log(doc)) {
        tallies
            .entry(doc.collection.clone())
            .or_default()
            .add(doc, now);
    }
    let mut ranked: Vec<(String, Tally)> = tallies.into_iter().collect();
    ranked.sort_by(|(left, a), (right, b)| {
        a.newest_age(now)
            .cmp(&b.newest_age(now))
            .then_with(|| left.cmp(right))
    });
    ranked
        .into_iter()
        .map(|(name, tally)| tally.stream(name))
        .collect()
}

/// What the entries of one stream add up to: how many there are, how many are new, and the newest stamp.
#[derive(Default)]
struct Tally {
    /// Every entry of the stream.
    count: usize,
    /// The entries that are new.
    new: usize,
    /// The newest stamp among the entries that have one.
    newest: Option<Timestamp>,
}

impl Tally {
    /// Adds one entry to the stream: it counts, it is new when its stamp is recent, and it is the newest when it is the latest.
    fn add(&mut self, doc: &Doc, now: Timestamp) {
        self.count += 1;
        let Some(time) = stamp_of(doc) else {
            return;
        };
        if now.seconds_since(time) <= NEW_SECONDS {
            self.new += 1;
        }
        if self
            .newest
            .is_none_or(|kept| now.seconds_since(kept) > now.seconds_since(time))
        {
            self.newest = Some(time);
        }
    }

    /// The age in seconds of the newest entry, which orders the streams: a stream with no stamp has the largest age.
    fn newest_age(&self, now: Timestamp) -> i64 {
        self.newest.map_or(i64::MAX, |time| now.seconds_since(time))
    }

    /// The stream named `name`, as the atlas prints it.
    fn stream(self, name: String) -> Stream {
        Stream {
            name,
            count: self.count,
            new_24h: self.new,
            newest: self.newest.map(Timestamp::rfc3339),
        }
    }
}

/// The instant that a log's file name starts with (design §4.3), or None when the name has no stamp.
fn stamp_of(doc: &Doc) -> Option<Timestamp> {
    clock::leading_stamp(&doc.id)
}

/// The number of entries in the store that address the agent (design §3.5 `to` line). It is zero when none does, since the line
/// shows at zero (INFO-ZERO-01). The agent is matched without regard to case, as the `to:` filter matches (design §3.2).
fn to_of(corpus: &Corpus, agent: String) -> ToCount {
    let count = corpus
        .all_docs()
        .filter(|doc| corpus.is_log(doc) && addressed(doc, &agent))
        .count();
    ToCount { agent, count }
}

/// True when the `to` list of a doc holds the agent, ignoring case.
fn addressed(doc: &Doc, agent: &str) -> bool {
    doc.fields.iter().any(|(key, value)| {
        key == "to"
            && matches!(value, Value::List(items) if items.iter().any(|item| item.eq_ignore_ascii_case(agent)))
    })
}

/// The nearest README at or above the cwd (design §3.5 `here` line), when the cwd is below the root. The folder is named by its
/// root-relative label, `.` for the root, and the README's one-liner is its title.
fn here_of(corpus: &Corpus, cwd: &Path) -> Option<HereLine> {
    let rel = slash(cwd.strip_prefix(&corpus.root).ok()?);
    if rel.is_empty() {
        return None;
    }
    let mut dir = rel.as_str();
    loop {
        if let Some(doc) = corpus
            .doc(&readme_label(dir))
            .filter(|doc| doc.collection == READMES)
        {
            return Some(HereLine {
                path: label_of(dir),
                title: doc.title.clone(),
            });
        }
        dir = parent_of(dir)?;
    }
}

/// One warning for each case id that more than one CASES.md declares, naming every file that declares it (design §4.2). The lines
/// come in id order.
fn duplicate_warnings(corpus: &Corpus) -> Vec<String> {
    let mut declared: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for doc in corpus
        .all_docs()
        .filter(|doc| doc.collection == CASES && !corpus.is_log(doc))
    {
        declared
            .entry(doc.id.as_str())
            .or_default()
            .push(doc.path.as_deref().unwrap_or_default());
    }
    declared
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(id, files)| {
            error::warning(&format!("duplicate case id {id} in {}", files.join(", ")))
        })
        .collect()
}
