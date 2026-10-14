//! Carries out `memex search` (design §3.2 and §3.6): gathers, groups and ranks docs, then prints.
//! Cases: SRCH-RANK-01, SRCH-RANK-02, SRCH-WEIGHT-01, SRCH-WEIGHT-02, SRCH-STEM-01, SRCH-PREFIX-01, SRCH-PREFIX-02, SRCH-FUZZY-01, SRCH-FUZZY-02, SRCH-TIE-01, SRCH-TOKEN-01, SRCH-TOKEN-02, SRCH-GROUP-01, SRCH-GROUP-02, SRCH-LIMIT-01, SRCH-IN-01, SRCH-IN-02, SRCH-RANK-03, SRCH-TOKEN-03, SRCH-JSON-01, SRCH-JSON-02, SRCH-LINE-01

use std::collections::BTreeMap;

use crate::clock::{self, Timestamp};
use crate::corpus::{Corpus, Doc};
use crate::error::Error;
use crate::index::tokenize::query_tokens;
use crate::index::{Index, Source};
use crate::render::search::{self as render, Group, Hit};
use crate::verbs::here;

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
    let corpus = here::load()?.corpus;
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
    let groups = rank_groups(&corpus, &plan);
    Ok(if request.json {
        render::document(&request.terms.join(" "), &groups)
    } else {
        render::text(&groups)
    })
}

/// The group of a doc: `logs` for an entry of the log store, whose streams are all one group, and its collection otherwise,
/// which is a built-in group or a manifest's name.
fn group_of(corpus: &Corpus, doc: &Doc) -> String {
    if corpus.is_log(doc) {
        LOGS.to_string()
    } else {
        doc.collection.clone()
    }
}

/// The groups that hold hits, in the order of design §3.2. A group that `--in` leaves out is not searched, and a group with no
/// hit is left out. With no words there are no hits.
fn rank_groups<'a>(corpus: &'a Corpus, plan: &Plan) -> Vec<Group<'a>> {
    if plan.words.is_empty() {
        return Vec::new();
    }
    let mut groups: BTreeMap<(u8, String), Vec<&'a Doc>> = BTreeMap::new();
    for doc in corpus.first_docs() {
        let name = group_of(corpus, doc);
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
