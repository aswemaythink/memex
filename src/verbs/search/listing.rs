//! Lists the docs of a `memex search` listing or roster group, newest logs first, cut to the limit.
//! A roster line carries its doc's cited-by count, and a log shows its age when its name has a stamp (design §3.2).
//! Cases: SRCH-AGE-01, SRCH-FILTER-01, SRCH-FILTER-06, SRCH-LIMIT-01, SRCH-ORPHAN-01, SRCH-ROSTER-01, SRCH-ROSTER-02, SRCH-ROSTER-04, SRCH-ROSTER-05, SRCH-EXCLUDE-03

use std::cmp::Ordering;

use crate::clock::{self, Timestamp};
use crate::corpus::{Corpus, Doc};
use crate::index::filter::time_of;
use crate::render::search::{self as render, Group, Hit};

/// The listing of a group: the docs that the filters keep, in listing order, with the limit applied. A roster's hits carry the count
/// of docs that cite each one. None when no doc is kept.
pub fn list_group<'a>(
    corpus: &Corpus,
    name: String,
    docs: Vec<&'a Doc>,
    limit: usize,
    now: Timestamp,
    roster: bool,
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
        .take(limit)
        .map(|(_, doc)| Hit {
            doc,
            score: None,
            fuzzy: Vec::new(),
            age: age_of(corpus, doc, now),
            cited_by: roster.then(|| corpus.citations().direct_count(&doc.label)),
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

/// The age of a log entry, from the stamp that starts its file name, which is the instant of its `time` field. Other docs have no
/// age, and nor does a log whose name has no stamp.
pub fn age_of(corpus: &Corpus, doc: &Doc, now: Timestamp) -> Option<String> {
    if !corpus.is_log(doc) {
        return None;
    }
    clock::leading_stamp(&doc.id).map(|stamp| render::age(now.seconds_since(stamp)))
}
