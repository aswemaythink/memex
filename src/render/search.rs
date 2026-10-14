//! Prints `search`'s groups as text (design §3.2) and as the `--json` document (design §3.6).
//! Cases: SRCH-GROUP-01, SRCH-GROUP-02, SRCH-LIMIT-01, SRCH-FUZZY-01, SRCH-LINE-01, SRCH-JSON-01, SRCH-JSON-02, SRCH-JSON-03, SRCH-FILTER-06, SRCH-ROSTER-01, SRCH-ROSTER-02, SRCH-ROSTER-03, SRCH-ROSTER-04, SRCH-ORPHAN-01

use std::collections::BTreeMap;

use serde_json::{json, Value as Json};

use super::text::clip;
use crate::collections::frontmatter::Value;
use crate::corpus::{Doc, LOGS};

/// The separator between the parts of a log's meta column (design §3.2).
const DOT: &str = " · ";

/// One group of hits: its name, how many docs matched in all, and the hits that are shown.
pub struct Group<'a> {
    /// The group's name: `readmes`, `headers`, `cases`, a manifest's name, or `logs`.
    pub name: String,
    /// How many docs of the group matched, before the limit.
    pub total: usize,
    /// The hits shown, best first.
    pub hits: Vec<Hit<'a>>,
}

/// One hit: its doc, its score, the index terms that matched by one edit, and the age of a log.
pub struct Hit<'a> {
    /// The doc that matched.
    pub doc: &'a Doc,
    /// The BM25F score, which the JSON document rounds to 3 decimals; None for a listing or a roster, which ranks nothing.
    pub score: Option<f64>,
    /// The index terms that matched by one edit, shown as `[~variant]`.
    pub fuzzy: Vec<String>,
    /// The age of a log entry, in the short form of `age`; None for any other doc, or for a log whose name has no stamp.
    pub age: Option<String>,
    /// The direct cited-by count of a roster's hit, which its line ends with as `<-N`; None for any other hit.
    pub cited_by: Option<usize>,
}

/// The text of the groups: each group's heading, then its hit lines. No groups give no text.
pub fn text(groups: &[Group]) -> String {
    groups.iter().map(group_text).collect()
}

/// The age of an entry in the short form of design §3.6: `Nm` below an hour, `Nh` below 48 hours, else `Nd`. A negative age
/// counts as zero.
pub fn age(seconds: i64) -> String {
    let seconds = seconds.max(0);
    if seconds < 3_600 {
        format!("{}m", seconds / 60)
    } else if seconds < 48 * 3_600 {
        format!("{}h", seconds / 3_600)
    } else {
        format!("{}d", seconds / 86_400)
    }
}

/// The `--json` document: the query, the filters, and the groups (design §3.6), with one line break at the end. Each filter's name
/// holds the list of its values, in the order given.
pub fn document(query: &str, filters: &[(&str, &str)], groups: &[Group]) -> String {
    let groups: Vec<Json> = groups.iter().map(group_json).collect();
    let mut by_name: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for &(name, value) in filters {
        by_name.entry(name).or_default().push(value);
    }
    format!(
        "{}\n",
        json!({"query": query, "filters": by_name, "groups": groups})
    )
}

/// One group's heading and hit lines, each line ending in a newline.
fn group_text(group: &Group) -> String {
    let mut out = format!(
        "{}  (showing {} of {})\n",
        group.name,
        group.hits.len(),
        group.total
    );
    for hit in &group.hits {
        out.push_str(&hit_line(hit, group.name == LOGS));
    }
    out
}

/// One hit's line: its label, then two spaces before each non-empty part. A log's parts start with its meta column; every
/// hit's parts end with its one-liner and its fuzzy markers.
fn hit_line(hit: &Hit, is_log: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    if is_log {
        parts.extend(log_meta(hit));
    }
    if !hit.doc.title.is_empty() {
        parts.push(clip(&hit.doc.title));
    }
    if !hit.fuzzy.is_empty() {
        parts.push(markers(&hit.fuzzy));
    }
    if let Some(cited) = hit.cited_by {
        parts.push(format!("<-{cited}"));
    }
    let mut line = format!("  {}", hit.doc.label);
    for part in parts {
        line.push_str("  ");
        line.push_str(&part);
    }
    line.push('\n');
    line
}

/// A log's meta column: its age, agent and branch, joined by ` · `, with a missing part left out with its separator. None when
/// no part is known.
fn log_meta(hit: &Hit) -> Option<String> {
    let parts: Vec<&str> = [
        hit.age.as_deref(),
        field(hit.doc, "agent"),
        field(hit.doc, "branch"),
    ]
    .into_iter()
    .flatten()
    .collect();
    (!parts.is_empty()).then(|| parts.join(DOT))
}

/// The fuzzy markers of a hit: each matched variant as `[~variant]`, separated by spaces.
fn markers(fuzzy: &[String]) -> String {
    let marks: Vec<String> = fuzzy
        .iter()
        .map(|variant| format!("[~{variant}]"))
        .collect();
    marks.join(" ")
}

/// The text of a frontmatter field that holds a string, or None when the doc has no such field.
fn field<'a>(doc: &'a Doc, key: &str) -> Option<&'a str> {
    doc.fields.iter().find_map(|(name, value)| match value {
        Value::Str(text) if name.as_str() == key => Some(text.as_str()),
        _ => None,
    })
}

/// One group as JSON: its name as the collection, its total, and its hits.
fn group_json(group: &Group) -> Json {
    let hits: Vec<Json> = group.hits.iter().map(hit_json).collect();
    json!({"collection": group.name, "total": group.total, "hits": hits})
}

/// One hit as JSON: the doc's ref, collection, id, path, title and fields, then its score rounded to 3 decimals (null in a listing
/// or a roster), its snippet (the full one-liner), its fuzzy variants, and for a roster hit its `cited_by` count.
fn hit_json(hit: &Hit) -> Json {
    let doc = hit.doc;
    let mut value = json!({
        "ref": doc.label,
        "collection": doc.collection,
        "id": doc.id,
        "path": doc.path,
        "title": doc.title,
        "fields": super::json::pairs(&doc.fields),
        "score": hit.score.map(round3),
        "snippet": doc.title,
        "fuzzy": hit.fuzzy,
    });
    if let Some(cited) = hit.cited_by {
        value["cited_by"] = json!(cited);
    }
    value
}

/// A score rounded to 3 decimals, as the JSON document prints it (design §3.6).
fn round3(score: f64) -> f64 {
    (score * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages_switch_units_at_an_hour_and_two_days() {
        assert_eq!(age(0), "0m");
        assert_eq!(age(59 * 60), "59m");
        assert_eq!(age(3_600), "1h");
        assert_eq!(age(47 * 3_600), "47h");
        assert_eq!(age(48 * 3_600), "2d");
        assert_eq!(age(-5), "0m");
    }
}
