//! Reads and matches a search's filters, and checks its `--in` names (design §3.2).
//! Cases: SRCH-FILTER-01, SRCH-FILTER-02, SRCH-FILTER-03, SRCH-FILTER-04, SRCH-FILTER-05, SRCH-FILTER-07, SRCH-FILTER-08, SRCH-IN-03, SRCH-IN-04, SRCH-IN-06, SRCH-LITERAL-01, SRCH-LITERAL-02, SRCH-LITERAL-03, SRCH-RESERVED-01, SRCH-SINCE-01, SRCH-SINCE-02, SRCH-SINCE-03, SRCH-SINCE-04, SRCH-JSON-03, CLI-HINT-03, CLI-HINT-04, SRCH-FOLD-01

use std::collections::BTreeSet;

use crate::cli::Verb;
use crate::clock::{self, Timestamp};
use crate::collections::frontmatter::Value;
use crate::corpus::{Corpus, Doc, CASES, HEADERS, LOGS, READMES};
use crate::error::Error;
use crate::scan::fail;

/// The names that are always filters, whether or not a doc in scope holds the field (design §3.2).
const RESERVED: [&str; 6] = ["agent", "to", "tag", "branch", "worktree", "since"];

/// The frontmatter field that `tag:` reads: a doc's tags are the items of its `tags` list.
const TAG_FIELD: &str = "tags";

/// One filter of a query: a field's value, or the bound of `since:`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// `name:value`: a doc whose field holds the value, as its text or as one item of its list.
    Field {
        /// The name as the query spells it, such as `tag`.
        name: String,
        /// The frontmatter field that it reads, such as `tags`.
        field: String,
        /// The value as the query spells it.
        value: String,
        /// The value lowercased as `str::to_lowercase` gives it, which a doc's item must lowercase to (design §3.2).
        folded: String,
    },
    /// `since:value`: a doc whose `time` is at or after the bound.
    Since {
        /// The value as the query spells it, such as `2d`.
        raw: String,
        /// What the value means.
        bound: Bound,
    },
}

/// The bound of `since:`: a span before the clock's instant, or the start of a UTC day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    /// A span of this many seconds before now, from `Nm`, `Nh`, `Nd` or `Nw`.
    Within(i64),
    /// The start of a `YYYY-MM-DD` day, which is UTC midnight.
    From(Timestamp),
}

impl Filter {
    /// The name and the value as the query spells them, for the JSON `filters` object. A `since:` filter is named `since`.
    pub fn spelled(&self) -> (&str, &str) {
        match self {
            Filter::Field { name, value, .. } => (name.as_str(), value.as_str()),
            Filter::Since { raw, .. } => ("since", raw.as_str()),
        }
    }

    /// True when the doc passes this filter, with `now` as the instant that `since:` is measured from.
    pub fn matches(&self, doc: &Doc, now: Timestamp) -> bool {
        match self {
            Filter::Field {
                name,
                field,
                folded,
                ..
            } => holds(doc, name, field, folded),
            Filter::Since { bound, .. } => time_of(doc).is_some_and(|at| bound.keeps(at, now)),
        }
    }
}

impl Bound {
    /// True when an instant is at or after the bound, given that `now` is the clock's instant.
    fn keeps(self, at: Timestamp, now: Timestamp) -> bool {
        match self {
            Bound::Within(seconds) => now.seconds_since(at) <= seconds,
            Bound::From(start) => at.seconds_since(start) >= 0,
        }
    }
}

/// Splits the query terms into filters and literal terms (design §3.2). A term is a filter when it reads `name:value`, with a
/// non-empty value, and `name` is reserved or some doc in scope holds that field. A reserved name with no value, such as `tag:`,
/// is a usage error, and every other term is returned as written, for its words. A `since:` value that is none of its forms is
/// a usage error too.
pub fn read(
    terms: &[String],
    scope: &BTreeSet<String>,
) -> Result<(Vec<Filter>, Vec<String>), Error> {
    let mut filters = Vec::new();
    let mut literals = Vec::new();
    for term in terms {
        match split(term) {
            Some((name, "")) if is_reserved(name) => return Err(empty_value(name)),
            Some((name, value)) if !value.is_empty() && is_filter(name, scope) => {
                filters.push(filter_of(name, value)?);
            }
            _ => literals.push(term.clone()),
        }
    }
    Ok((filters, literals))
}

/// The names of the fields that the docs hold, which decide what `name:value` means in a search over those docs.
pub fn fields_of<'a>(docs: impl Iterator<Item = &'a Doc>) -> BTreeSet<String> {
    docs.flat_map(|doc| doc.fields.iter().map(|(name, _)| name.clone()))
        .collect()
}

/// Checks the names given to `--in` (design §3.2): each is a built-in collection (`readmes`, `headers`, `cases` or `logs`), a
/// collection or stream that a doc of the corpus holds, or a manifest that loads. The first name that is none of these is a usage
/// error, so an unknown name never gives an empty result.
pub fn check_in(corpus: &Corpus, names: &[String]) -> Result<(), Error> {
    let held: BTreeSet<&str> = corpus
        .all_docs()
        .map(|doc| doc.collection.as_str())
        .collect();
    for name in names {
        let builtin = crate::corpus::RESERVED.contains(&name.as_str());
        let manifest = corpus
            .manifests
            .iter()
            .any(|found| found.manifest.name == *name);
        if !(builtin || held.contains(name.as_str()) || manifest) {
            let listed = valid_names(corpus);
            let message = format!("unknown collection '{name}'; collections: {listed}");
            return Err(fail(Verb::Search, message));
        }
    }
    Ok(())
}

/// The names that `--in` takes, in the order design §3.5c gives: readmes, headers and cases, the manifests in name order, logs,
/// then the streams in name order. A stream is a log doc's collection that no manifest takes, so each name is listed once.
fn valid_names(corpus: &Corpus) -> String {
    let manifests: BTreeSet<&str> = corpus
        .manifests
        .iter()
        .map(|found| found.manifest.name.as_str())
        .collect();
    let streams: BTreeSet<&str> = corpus
        .all_docs()
        .filter(|doc| corpus.is_log(doc))
        .map(|doc| doc.collection.as_str())
        .filter(|name| !manifests.contains(name))
        .collect();
    let mut names = vec![READMES, HEADERS, CASES];
    names.extend(manifests.iter().copied());
    names.push(LOGS);
    names.extend(streams);
    names.join(", ")
}

/// The instant of a doc's `time` field, which is written as `YYYY-MM-DDTHH:MM:SSZ` (design §3.6). None for a doc with no time, and
/// for a time in any other form.
pub fn time_of(doc: &Doc) -> Option<Timestamp> {
    let text = doc.fields.iter().find_map(|(name, value)| match value {
        Value::Str(text) if name.as_str() == "time" => Some(text.as_str()),
        _ => None,
    })?;
    clock::parse_rfc3339(text)
}

/// The name and the value of a term that reads `name:value`: split at the first colon, with a non-empty name of letters, digits,
/// dashes and underscores. The value may be empty. None for any other term.
fn split(term: &str) -> Option<(&str, &str)> {
    let (name, value) = term.split_once(':')?;
    let name_ok = !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    name_ok.then_some((name, value))
}

/// True when a name is one of the reserved names, ignoring ASCII case.
fn is_reserved(name: &str) -> bool {
    RESERVED
        .iter()
        .any(|reserved| reserved.eq_ignore_ascii_case(name))
}

/// True when a name is reserved, or some doc in scope holds a field of that name. Names compare ignoring ASCII case.
fn is_filter(name: &str, scope: &BTreeSet<String>) -> bool {
    is_reserved(name) || scope.iter().any(|field| field.eq_ignore_ascii_case(name))
}

/// The usage error of a reserved name with no value, such as `tag:`, which must not be searched as the word `tag` (design §3.2).
fn empty_value(name: &str) -> Error {
    fail(Verb::Search, format!("empty filter value '{name}:'"))
}

/// The filter that `name:value` names. `since:` reads its bound, `tag:` reads the `tags` field, and any other name reads the field
/// of its own name.
fn filter_of(name: &str, value: &str) -> Result<Filter, Error> {
    if name.eq_ignore_ascii_case("since") {
        let bound = bound_of(value)?;
        return Ok(Filter::Since {
            raw: value.to_string(),
            bound,
        });
    }
    let field = if name.eq_ignore_ascii_case("tag") {
        TAG_FIELD
    } else {
        name
    };
    Ok(Filter::Field {
        name: name.to_string(),
        field: field.to_string(),
        value: value.to_string(),
        folded: value.to_lowercase(),
    })
}

/// The bound of a `since:` value: `Nm`, `Nh`, `Nd` or `Nw`, in the duration grammar of design §4.3, or a `YYYY-MM-DD` date.
fn bound_of(value: &str) -> Result<Bound, Error> {
    let bound = date_of(value)
        .map(Bound::From)
        .or_else(|| clock::parse_duration(value).map(Bound::Within));
    bound.ok_or_else(|| {
        let message =
            format!("bad value '{value}' for 'since:': expected Nm, Nh, Nd, Nw or YYYY-MM-DD");
        fail(Verb::Search, message)
    })
}

/// The instant of UTC midnight at the start of a `YYYY-MM-DD` date. None when the text has any other shape or is not a real date.
fn date_of(text: &str) -> Option<Timestamp> {
    let shaped = text.len() == 10
        && text.bytes().enumerate().all(|(at, byte)| match at {
            4 | 7 => byte == b'-',
            _ => byte.is_ascii_digit(),
        });
    if !shaped {
        return None;
    }
    clock::parse_stamp(&format!(
        "{}{}{}-000000",
        &text[..4],
        &text[5..7],
        &text[8..]
    ))
}

/// True when the doc holds the field, and one of its items (its text, or an item of its list) matches the filter's folded value.
fn holds(doc: &Doc, name: &str, field: &str, folded: &str) -> bool {
    doc.fields.iter().any(|(key, found)| {
        key.eq_ignore_ascii_case(field) && items(found).iter().any(|item| same(name, item, folded))
    })
}

/// The items of a field: its text as the one item, or each item of its list.
fn items(found: &Value) -> Vec<&str> {
    match found {
        Value::Str(text) => vec![text.as_str()],
        Value::List(list) => list.iter().map(String::as_str).collect(),
    }
}

/// True when an item matches a filter's folded value (design §3.2). A `worktree` value with no `/` matches the basename of the
/// item, and every other value matches the whole item.
fn same(name: &str, item: &str, folded: &str) -> bool {
    if name.eq_ignore_ascii_case("worktree") && !folded.contains('/') {
        let basename = item.rsplit('/').next().unwrap_or(item);
        return same_text(basename, folded);
    }
    same_text(item, folded)
}

/// True when an item, lowercased as `str::to_lowercase` gives it, equals a folded value. An ASCII item lowercases to ASCII, so it
/// cannot equal a folded value that is not ASCII. ASCII pairs compare in place, so only a non-ASCII item allocates.
fn same_text(item: &str, folded: &str) -> bool {
    match (item.is_ascii(), folded.is_ascii()) {
        (true, true) => item.eq_ignore_ascii_case(folded),
        (true, false) => false,
        (false, _) => item.to_lowercase() == folded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_term_reads_as_a_name_and_a_value_only_with_a_name() {
        assert_eq!(split("tag:show"), Some(("tag", "show")));
        assert_eq!(split("std::io"), Some(("std", ":io")));
        assert_eq!(split("tag:"), Some(("tag", "")));
        assert_eq!(split(":show"), None);
        assert_eq!(split("no colon"), None);
        assert_eq!(split("a.b:c"), None);
    }

    #[test]
    fn a_reserved_name_with_no_value_is_a_usage_error() {
        let error = read(&["since:".to_string()], &BTreeSet::new()).unwrap_err();
        assert_eq!(error.message, "empty filter value 'since:'");
        let error = read(&["TAG:".to_string()], &BTreeSet::new()).unwrap_err();
        assert_eq!(error.message, "empty filter value 'TAG:'");
    }

    #[test]
    fn dates_are_utc_midnight_and_refuse_other_shapes() {
        assert_eq!(date_of("2026-10-12"), clock::parse_stamp("20261012-000000"));
        assert_eq!(date_of("2026-02-30"), None);
        assert_eq!(date_of("2026-10-1"), None);
        assert_eq!(date_of("2026/10/12"), None);
    }

    #[test]
    fn a_worktree_matches_its_basename_or_its_full_path() {
        assert!(same("worktree", "/work/wt-cook-2", "wt-cook-2"));
        assert!(same("worktree", "/work/wt-cook-2", "/work/wt-cook-2"));
        assert!(!same("worktree", "/work/wt-cook-2", "/wt-cook-2"));
    }

    #[test]
    fn values_match_as_unicode_lowercasing_gives_them() {
        let fold = |value: &str| value.to_lowercase();
        assert!(same("tag", "café", &fold("CAFÉ")));
        assert!(!same("tag", "cafe", &fold("CAFÉ")));
        // Kelvin sign lowercases to an ASCII k, and a final sigma lowercases to ς at the end of a word.
        assert!(same("tag", "k", &fold("\u{212a}")));
        assert!(same("tag", "οδος", &fold("ΟΔΟΣ")));
        assert!(same("worktree", "/work/Café", &fold("café")));
    }
}
