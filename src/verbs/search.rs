//! Carries out `memex search` (design §3.2 and §3.6): filters, then ranks, lists or rosters docs.
//! Cases: SRCH-RANK-01, SRCH-RANK-02, SRCH-WEIGHT-01, SRCH-WEIGHT-02, SRCH-STEM-01, SRCH-PREFIX-01, SRCH-PREFIX-02, SRCH-FUZZY-01, SRCH-FUZZY-02, SRCH-TIE-01, SRCH-TOKEN-01, SRCH-TOKEN-02, SRCH-GROUP-01, SRCH-GROUP-02, SRCH-LIMIT-01, SRCH-IN-01, SRCH-IN-02, SRCH-RANK-03, SRCH-TOKEN-03, SRCH-JSON-01, SRCH-JSON-02, SRCH-LINE-01, SRCH-JSON-03, SRCH-FILTER-01, SRCH-FILTER-02, SRCH-FILTER-03, SRCH-FILTER-04, SRCH-FILTER-05, SRCH-FILTER-06, SRCH-LITERAL-01, SRCH-LITERAL-02, SRCH-LITERAL-03, SRCH-RESERVED-01, SRCH-ROSTER-01, SRCH-ROSTER-02, SRCH-ROSTER-03, SRCH-ROSTER-04, SRCH-ORPHAN-01, SRCH-ORPHAN-02, SRCH-SINCE-01, SRCH-SINCE-02, SRCH-SINCE-03, SRCH-SINCE-04, SRCH-IN-03, SRCH-IN-04, SRCH-IN-05, SRCH-IN-06, SRCH-TOKEN-04, HDR-CAP-03, MAN-LOAD-13, SRCH-NOHITS-01, SRCH-NOHITS-02, SRCH-NOHITS-03

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::cli::Verb;
use crate::clock::{self, Timestamp};
use crate::corpus::{Corpus, Doc, CASES, HEADERS, LOGS, READMES};
use crate::error::{warn, Error};
use crate::index::bm25;
use crate::index::filter::{self, time_of, Filter};
use crate::index::tokenize::query_tokens;
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
    /// Words are given: the docs that the filters keep are ranked by the words.
    Words,
    /// Filters are given with no words: the docs that they keep are listed.
    Listing,
    /// `--in` is given with no words: each group's docs are listed, with the count of docs that cite each one.
    Roster,
    /// No words, filters or `--in` are given: nothing is listed.
    Nothing,
}

/// What a search asks of the index: its mode, its words and filters, the limit of each group, and the instant that log ages and
/// `since:` are measured from.
struct Plan {
    mode: Mode,
    words: Vec<String>,
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
    let (filters, literals) = filter::read(&request.terms, &fields)?;
    let plan = Plan::new(request, &literals, filters, now)?;
    let groups = rank_groups(&corpus, &plan, scope);
    if groups.is_empty() {
        warn(&render::no_hits(&request.terms, &literals));
    }
    Ok(if request.json {
        let spelled: Vec<(&str, &str)> = plan.filters.iter().map(Filter::spelled).collect();
        render::document(&request.terms.join(" "), &spelled, &groups)
    } else {
        render::text(&groups)
    })
}

impl Plan {
    /// The plan of a request: its words are the tokens of its literal terms, its mode follows from what is given, and its limit is
    /// the `--limit`, else the default of the mode. Literal terms that hold no token are a usage error, not an empty search.
    fn new(
        request: &Request,
        literals: &[String],
        filters: Vec<Filter>,
        now: Timestamp,
    ) -> Result<Plan, Error> {
        let words = words_of(literals);
        if words.is_empty() && !literals.is_empty() {
            return Err(fail(Verb::Search, "no searchable words".to_string()));
        }
        let mode = mode_of(
            !words.is_empty(),
            request.in_.is_some(),
            !filters.is_empty(),
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

/// The mode of a query: words rank the docs; otherwise `--in` makes a roster; otherwise filters list the docs they keep.
fn mode_of(has_words: bool, in_given: bool, has_filters: bool) -> Mode {
    match (has_words, in_given, has_filters) {
        (true, _, _) => Mode::Words,
        (false, true, _) => Mode::Roster,
        (false, false, true) => Mode::Listing,
        (false, false, false) => Mode::Nothing,
    }
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

/// The groups that the search reads, each with its docs, in the order of design §3.2. `--in` leaves out the docs that it does not
/// name, and a group with no doc is left out.
fn scope_of<'a>(corpus: &'a Corpus, in_: Option<&[String]>) -> Vec<(String, Vec<&'a Doc>)> {
    let mut groups: BTreeMap<(u8, String), Vec<&'a Doc>> = BTreeMap::new();
    for doc in corpus.first_docs().filter(|doc| wanted(corpus, in_, doc)) {
        let name = group_of(corpus, doc);
        groups.entry((rank(&name), name)).or_default().push(doc);
    }
    groups
        .into_iter()
        .map(|((_, name), docs)| (name, docs))
        .collect()
}

/// True when there is no `--in` list, or when the list names the doc's group. A log entry's group is `logs`, which `logs` names, and
/// its stream, which a stream name names: so `--in impl` keeps only the entries of stream `impl` (design §3.2).
fn wanted(corpus: &Corpus, in_: Option<&[String]>, doc: &Doc) -> bool {
    let Some(names) = in_ else {
        return true;
    };
    let group = group_of(corpus, doc);
    names
        .iter()
        .any(|named| *named == group || (group == LOGS && *named == doc.collection))
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

/// The groups that hold hits or listed docs, in the order of design §3.2. Each group's docs are first narrowed by the filters, then
/// ranked by the words, or listed. A query that asks for nothing has no groups.
fn rank_groups<'a>(
    corpus: &Corpus,
    plan: &Plan,
    scope: Vec<(String, Vec<&'a Doc>)>,
) -> Vec<Group<'a>> {
    if plan.mode == Mode::Nothing {
        return Vec::new();
    }
    scope
        .into_iter()
        .filter_map(|(name, docs)| {
            let kept: Vec<&'a Doc> = docs.into_iter().filter(|doc| plan.keeps(doc)).collect();
            match plan.mode {
                Mode::Words => search_group(corpus, name, &kept, plan),
                _ => list_group(corpus, name, kept, plan),
            }
        })
        .collect()
}

/// The field weights of a group: its manifest's `weights`, or the defaults for a group with no manifest (design §4.2).
fn weights_of(corpus: &Corpus, name: &str) -> [f64; 3] {
    corpus
        .manifests
        .iter()
        .find(|found| found.manifest.name == name)
        .map_or(bm25::WEIGHTS, |found| {
            bm25::field_weights(&found.manifest.weights)
        })
}

/// The group of docs that the words rank: its total, and its hits within the limit. None when no doc matches.
fn search_group<'a>(
    corpus: &Corpus,
    name: String,
    docs: &[&'a Doc],
    plan: &Plan,
) -> Option<Group<'a>> {
    let weights = weights_of(corpus, &name);
    let sources: Vec<Source> = docs.iter().map(|doc| source_of(doc)).collect();
    let hits = Index::build(&sources, weights).search(&plan.words);
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
                age: age_of(&name, doc, plan.now),
                cited_by: None,
            }
        })
        .collect();
    Some(Group {
        name,
        total,
        hits: shown,
    })
}

/// The listing of a group: the docs that the filters keep, in listing order, with the limit applied. A roster's hits carry the count
/// of docs that cite each one. None when no doc is kept.
fn list_group<'a>(
    corpus: &Corpus,
    name: String,
    docs: Vec<&'a Doc>,
    plan: &Plan,
) -> Option<Group<'a>> {
    if docs.is_empty() {
        return None;
    }
    let total = docs.len();
    let mut keyed: Vec<(Option<Timestamp>, &'a Doc)> = docs
        .into_iter()
        .map(|doc| (listed_time(corpus, doc), doc))
        .collect();
    keyed.sort_by(|left, right| {
        compare_newest(left.0, right.0).then_with(|| left.1.label.cmp(&right.1.label))
    });
    let hits = keyed
        .into_iter()
        .take(plan.limit)
        .map(|(_, doc)| Hit {
            doc,
            score: None,
            fuzzy: Vec::new(),
            age: age_of(&name, doc, plan.now),
            cited_by: (plan.mode == Mode::Roster)
                .then(|| corpus.citations().direct_count(&doc.label)),
        })
        .collect();
    Some(Group { name, total, hits })
}

/// The time that orders a listed doc: a log's `time`. Every other doc has none, so the docs of a group are ordered by label.
fn listed_time(corpus: &Corpus, doc: &Doc) -> Option<Timestamp> {
    if corpus.is_log(doc) {
        time_of(doc)
    } else {
        None
    }
}

/// The order of two listed times, newest first: a later time comes before an earlier one, and a doc with no time comes last.
fn compare_newest(left: Option<Timestamp>, right: Option<Timestamp>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.seconds_since(left).cmp(&0),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// A doc as the index reads it. A header is read without its cut note and cut to its first `HEADER_LINES` lines; the text of every
/// other doc is read whole.
fn source_of(doc: &Doc) -> Source<'_> {
    let body = if doc.collection == HEADERS {
        first_lines(header::without_note(&doc.text), HEADER_LINES)
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
    clock::leading_stamp(&doc.id).map(|stamp| render::age(now.seconds_since(stamp)))
}

/// The words of a query: every token of every literal term in order, with no repeats. A term that holds no token adds none.
fn words_of(terms: &[String]) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for token in terms.iter().flat_map(|term| query_tokens(term)) {
        if !words.contains(&token) {
            words.push(token);
        }
    }
    words
}
