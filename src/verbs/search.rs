//! Carries out `memex search` (design §3.2 and §3.6): filters, then ranks, lists or rosters docs.
//! Each doc is grouped by where it came from: the graph's READMEs, its headers, a collection, or the logs (design §3.2 and §4.2 Names).
//! Cases: SRCH-RANK-01, SRCH-RANK-02, SRCH-WEIGHT-01, SRCH-WEIGHT-02, SRCH-STEM-01, SRCH-PREFIX-01, SRCH-PREFIX-02, SRCH-FUZZY-01, SRCH-FUZZY-02, SRCH-TIE-01, SRCH-TOKEN-01, SRCH-TOKEN-02, SRCH-GROUP-01, SRCH-GROUP-02, SRCH-LIMIT-01, SRCH-IN-01, SRCH-IN-02, SRCH-RANK-03, SRCH-TOKEN-03, SRCH-JSON-01, SRCH-JSON-02, SRCH-LINE-01, SRCH-JSON-03, SRCH-FILTER-01, SRCH-FILTER-02, SRCH-FILTER-03, SRCH-FILTER-04, SRCH-FILTER-05, SRCH-FILTER-06, SRCH-LITERAL-01, SRCH-LITERAL-02, SRCH-LITERAL-03, SRCH-RESERVED-01, SRCH-ROSTER-01, SRCH-ROSTER-02, SRCH-ROSTER-03, SRCH-ROSTER-04, SRCH-ORPHAN-01, SRCH-ORPHAN-02, SRCH-SINCE-01, SRCH-SINCE-02, SRCH-SINCE-03, SRCH-SINCE-04, SRCH-IN-03, SRCH-IN-04, SRCH-IN-05, SRCH-IN-06, SRCH-TOKEN-04, HDR-CAP-03, MAN-LOAD-13, SRCH-NOHITS-01, SRCH-NOHITS-02, SRCH-NOHITS-03, SRCH-NOHITS-04, COLL-SHARED-01, COLL-SHARED-02, COLL-SHARED-03, SRCH-REQUIRE-01, SRCH-REQUIRE-02, SRCH-REQUIRE-03, SRCH-REQUIRE-04, SRCH-EXCLUDE-01, SRCH-EXCLUDE-02, SRCH-EXCLUDE-03, SRCH-EXCLUDE-04, SRCH-EXCLUDE-05, SRCH-EXCLUDE-06

mod listing;
mod query;

use std::collections::BTreeMap;

use self::listing::{age_of, list_group};
use self::query::{Literal, Words};
use crate::cli::Verb;
use crate::clock::Timestamp;
use crate::collections::frontmatter::after_frontmatter;
use crate::corpus::{Corpus, Doc, HEADERS, LOGS, READMES};
use crate::error::{warn, Error};
use crate::index::bm25;
use crate::index::filter::{self, Filter};
use crate::index::{Index, Source};
use crate::intent::header;
use crate::render::search::{self as render, Group, Hit};
use crate::scan::fail;
use crate::verbs::here;

/// The `--limit` of each group when `--in` is not given (design §3.2).
const DEFAULT_LIMIT: usize = 5;

/// The `--limit` of each group when `--in` is given with words (design §3.2).
const IN_LIMIT: usize = 20;

/// The `--limit` of each group of a roster, which is `--in` with no words (design §3.2).
const ROSTER_LIMIT: usize = 50;

/// The lines of a header that the index reads; `show` prints the whole header (design §2.1).
const HEADER_LINES: usize = 60;

/// The arguments of `memex search` once the grammar has passed them.
pub struct Request {
    /// The words and filters, in the order given.
    pub terms: Vec<String>,
    /// The groups named by `--in`, if the flag was given.
    pub in_: Option<Vec<String>>,
    /// The per-group `--limit`, if the flag was given.
    pub limit: Option<usize>,
    /// True when `--json` was given.
    pub json: bool,
}

/// What a query asks for (design §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Words that rank the docs are given: the docs that the filters keep are ranked by them, and the `+` and `!` words cut the hits.
    Words,
    /// Filters or `!` words are given with no ranking word: the docs that they keep are listed.
    Listing,
    /// `--in` is given with no ranking word: each group's docs are listed, with the count of docs that cite each one.
    Roster,
    /// No words, filters or `--in` are given: nothing is listed.
    Nothing,
}

/// What a search asks of the index: its mode, its words with their operators, its filters, the limit of each group, and the instant
/// that log ages and `since:` are measured from.
struct Plan {
    mode: Mode,
    words: Words,
    filters: Vec<Filter>,
    limit: usize,
    now: Timestamp,
}

/// Carries out a search and returns its stdout: the text of its groups, or the JSON document with `--json`. Zero hits give no
/// text, a hint on stderr (design §3.2), and the exit status is 0 as usual.
pub fn run(request: &Request, now: Timestamp) -> Result<String, Error> {
    let corpus = here::load()?.corpus;
    if let Some(names) = &request.in_ {
        filter::check_in(&corpus, names)?;
    }
    let scope = scope_of(&corpus, request.in_.as_deref());
    let fields = filter::fields_of(scope.iter().flat_map(|(_, docs)| docs.iter().copied()));
    let (filters, literals) = query::read(&request.terms, &fields)?;
    let plan = Plan::new(request, &literals, filters, now)?;
    let groups = rank_groups(&corpus, &plan, scope);
    if groups.is_empty() {
        warn(&render::no_hits(&request.terms, &typed(&literals)));
    }
    Ok(if request.json {
        let spelled: Vec<(String, &str)> = plan.filters.iter().map(Filter::spelled).collect();
        render::document(&request.terms.join(" "), &spelled, &groups)
    } else {
        render::text(&groups)
    })
}

/// The literal terms as typed, which the zero-hit hint names (design §3.2 Zero hits).
fn typed(literals: &[Literal]) -> Vec<String> {
    literals
        .iter()
        .map(|literal| literal.typed.clone())
        .collect()
}

impl Plan {
    /// The plan of a request: its words are read from its literal terms, its mode follows from what is given, and its limit is the
    /// `--limit`, else the default of the mode. Literal terms that hold no token are a usage error, not an empty search.
    fn new(
        request: &Request,
        literals: &[Literal],
        filters: Vec<Filter>,
        now: Timestamp,
    ) -> Result<Plan, Error> {
        let words = Words::of(literals);
        if words.is_empty() && !literals.is_empty() {
            return Err(fail(Verb::Search, "no searchable words".to_string()));
        }
        let mode = mode_of(
            !words.ranked.is_empty(),
            request.in_.is_some(),
            !filters.is_empty() || !words.excluded.is_empty(),
        );
        let default = match mode {
            Mode::Words if request.in_.is_some() => IN_LIMIT,
            Mode::Roster => ROSTER_LIMIT,
            _ => DEFAULT_LIMIT,
        };
        Ok(Plan {
            mode,
            words,
            filters,
            limit: request.limit.unwrap_or(default),
            now,
        })
    }

    /// True when the doc passes every filter of the query.
    fn keeps(&self, doc: &Doc) -> bool {
        self.filters
            .iter()
            .all(|filter| filter.matches(doc, self.now))
    }
}

/// The mode of a query: words that rank the docs make a search; otherwise `--in` makes a roster; otherwise filters or `!` words
/// make a listing of the docs they keep.
fn mode_of(ranked: bool, in_given: bool, listed: bool) -> Mode {
    match (ranked, in_given, listed) {
        (true, _, _) => Mode::Words,
        (false, true, _) => Mode::Roster,
        (false, false, true) => Mode::Listing,
        (false, false, false) => Mode::Nothing,
    }
}

/// The group of a doc (design §3.2 and §4.2 Names). A collection that shares a graph or log name is its own group, so the group is
/// told apart by where the doc came from, and its name is only what the heading shows.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum GroupKey {
    /// The graph's READMEs, which come first.
    Readmes,
    /// The graph's headers, which come after the READMEs.
    Headers,
    /// A collection, by its name, in name order, after the graph's groups.
    Collection(String),
    /// Every log entry, whose streams are one group, after the collections.
    Logs,
}

impl GroupKey {
    /// The name that the group's heading and its JSON `collection` show.
    fn name(&self) -> &str {
        match self {
            GroupKey::Readmes => READMES,
            GroupKey::Headers => HEADERS,
            GroupKey::Collection(name) => name.as_str(),
            GroupKey::Logs => LOGS,
        }
    }
}

/// The group of a doc: the logs for a log entry, the graph's READMEs or headers for those, and its collection otherwise.
fn key_of(corpus: &Corpus, doc: &Doc) -> GroupKey {
    if corpus.is_log(doc) {
        GroupKey::Logs
    } else if doc.is_readme() {
        GroupKey::Readmes
    } else if doc.is_header() {
        GroupKey::Headers
    } else {
        GroupKey::Collection(doc.collection.clone())
    }
}

/// The groups that the search reads, each with its docs, in the order of design §3.2. `--in` leaves out the docs that it does not
/// name, and a group with no doc is left out.
fn scope_of<'a>(corpus: &'a Corpus, in_: Option<&[String]>) -> Vec<(GroupKey, Vec<&'a Doc>)> {
    let mut groups: BTreeMap<GroupKey, Vec<&'a Doc>> = BTreeMap::new();
    for doc in corpus.first_docs().filter(|doc| wanted(corpus, in_, doc)) {
        groups.entry(key_of(corpus, doc)).or_default().push(doc);
    }
    groups.into_iter().collect()
}

/// True when there is no `--in` list, or when one of its names means the doc (design §3.2).
fn wanted(corpus: &Corpus, in_: Option<&[String]>, doc: &Doc) -> bool {
    let Some(names) = in_ else {
        return true;
    };
    names.iter().any(|name| names_doc(corpus, name, doc))
}

/// True when an `--in` name means the doc (design §3.2 and §4.2 Names): `readmes` and `headers` name the graph's, `logs` every log
/// entry, and a stream's name that stream's entries, even when a collection shares the name. Any other name is a collection's docs.
fn names_doc(corpus: &Corpus, name: &str, doc: &Doc) -> bool {
    match name {
        READMES => doc.is_readme(),
        HEADERS => doc.is_header(),
        LOGS => corpus.is_log(doc),
        _ if corpus.is_stream(name) => corpus.is_log(doc) && doc.collection == name,
        _ => !corpus.is_log(doc) && !doc.is_graph() && doc.collection == name,
    }
}

/// The groups that hold hits or listed docs, in the order of design §3.2. Each group's docs are first narrowed by the filters, then
/// ranked by the words, or listed with the `!` words left out. A query that asks for nothing has no groups.
fn rank_groups<'a>(
    corpus: &Corpus,
    plan: &Plan,
    scope: Vec<(GroupKey, Vec<&'a Doc>)>,
) -> Vec<Group<'a>> {
    if plan.mode == Mode::Nothing {
        return Vec::new();
    }
    let roster = plan.mode == Mode::Roster;
    scope
        .into_iter()
        .filter_map(|(key, docs)| {
            let kept: Vec<&'a Doc> = docs.into_iter().filter(|doc| plan.keeps(doc)).collect();
            match plan.mode {
                Mode::Words => search_group(corpus, &key, &kept, plan),
                _ => {
                    let listed = leave_out(kept, &plan.words.excluded);
                    list_group(
                        corpus,
                        key.name().to_string(),
                        listed,
                        plan.limit,
                        plan.now,
                        roster,
                    )
                }
            }
        })
        .collect()
}

/// The docs of a listing that its `!` words leave in: those that hold none of them at tier 0 (design §3.2 Operators). A listing with
/// no `!` word keeps every doc and builds no index.
fn leave_out<'a>(docs: Vec<&'a Doc>, excluded: &[String]) -> Vec<&'a Doc> {
    if excluded.is_empty() {
        return docs;
    }
    let pass = {
        let sources: Vec<Source> = docs.iter().map(|doc| source_of(doc)).collect();
        Index::build(&sources, bm25::WEIGHTS).passing(&[], excluded)
    };
    docs.into_iter()
        .zip(pass)
        .filter_map(|(doc, pass)| pass.then_some(doc))
        .collect()
}

/// The field weights of a group: its collection's manifest `weights`, or the defaults for the graph and the logs and for a
/// collection with no manifest (design §4.2).
fn weights_of(corpus: &Corpus, key: &GroupKey) -> [f64; 3] {
    let GroupKey::Collection(name) = key else {
        return bm25::WEIGHTS;
    };
    corpus
        .manifests
        .iter()
        .find(|found| found.manifest.name == *name)
        .map_or(bm25::WEIGHTS, |found| {
            bm25::field_weights(&found.manifest.weights)
        })
}

/// The group of docs that the words rank: its total, and its hits within the limit. The `+` and `!` words cut the hits down to those
/// that pass them, and the total counts those. None when no doc passes.
fn search_group<'a>(
    corpus: &Corpus,
    key: &GroupKey,
    docs: &[&'a Doc],
    plan: &Plan,
) -> Option<Group<'a>> {
    let sources: Vec<Source> = docs.iter().map(|doc| source_of(doc)).collect();
    let index = Index::build(&sources, weights_of(corpus, key));
    let pass = index.passing(&plan.words.required, &plan.words.excluded);
    let hits: Vec<_> = index
        .search(&plan.words.ranked)
        .into_iter()
        .filter(|hit| pass[hit.doc])
        .collect();
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
                score: Some(hit.score),
                fuzzy: hit.fuzzy,
                age: age_of(corpus, doc, plan.now),
                cited_by: None,
            }
        })
        .collect();
    Some(Group {
        name: key.name().to_string(),
        total,
        hits: shown,
    })
}

/// A doc as the index reads it. A header is read without its cut note and cut to its first `HEADER_LINES` lines, a README without a
/// closed frontmatter block at its top (design §2 One-liners); the text of every other doc is read whole.
fn source_of(doc: &Doc) -> Source<'_> {
    let body = if doc.is_header() {
        first_lines(header::without_note(&doc.text), HEADER_LINES)
    } else if doc.is_readme() {
        after_frontmatter(&doc.text)
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
