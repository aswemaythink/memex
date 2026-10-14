//! Prints `info`'s atlas as text under the 3,000-byte cap (design §3.5) and as JSON (design §3.6).
//! Cases: INFO-HEAD-01, INFO-HEAD-02, INFO-INTENT-01, INFO-INTENT-02, INFO-INTENT-03, INFO-COLL-01, INFO-COLL-02, INFO-STREAM-01, INFO-STREAM-02, INFO-STREAM-03, INFO-STREAM-04, INFO-TO-01, INFO-TO-02, INFO-TO-03, INFO-TO-04, INFO-TO-05, INFO-HERE-01, INFO-HERE-02, INFO-HERE-03, INFO-HERE-04, INFO-USE-01, INFO-CAP-01, INFO-CAP-02, INFO-WARN-01, INFO-WARN-02, INFO-NOROOT-01, INFO-NOROOT-02, INFO-JSON-01, INFO-JSON-02, INFO-JSON-03

use serde_json::{json, Value as Json};

use super::text::clip;

/// The most bytes that the text of the atlas may take (design §3.5).
const CAP: usize = 3_000;

/// The separator between the parts of a line (design §3.2 and §3.5).
const DOT: &str = " · ";

/// The use line, which names the three commands that the atlas points to (design §3.5).
const USE: &str = "memex show <path|ref> · memex search <words> · memex log <stream> < body";

/// The atlas that `info` prints: where things are, what is in them, and the warnings of the run (design §3.5 and §3.6).
pub struct Atlas {
    /// The root as an absolute path, with `/` separators.
    pub root: String,
    /// The log store as an absolute path, with `/` separators; None when there is no `.memex/`.
    pub store: Option<String>,
    /// The store as the header prints it: root-relative with a trailing `/` inside the root, else absolute, or `none`.
    pub store_label: String,
    /// The built-in docs of the intent layer, counted.
    pub intent: Intent,
    /// The manifest collections, in name order.
    pub collections: Vec<Collection>,
    /// The log streams, newest entry first and then by name.
    pub streams: Vec<Stream>,
    /// The entries addressed to the agent, when an agent is known and has some.
    pub to: Option<ToCount>,
    /// The nearest README at or above the cwd, when the cwd is below the root and such a README exists.
    pub here: Option<HereLine>,
    /// The warnings of the run: the manifests that did not load, the duplicate case ids, then the files that could not be read.
    /// Only JSON lists them.
    pub warnings: Vec<String>,
}

/// The counts of the built-in collections: READMEs, headers and case ids.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Intent {
    /// The READMEs of the root.
    pub readmes: usize,
    /// The files with a header block.
    pub headers: usize,
    /// The case ids of every CASES.md.
    pub cases: usize,
}

/// A manifest collection: its name, its kind, its doc count and what its manifest says about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    /// The collection's name, which is its manifest's file stem.
    pub name: String,
    /// The manifest's kind: `markdown`, `folder` or `json`.
    pub kind: &'static str,
    /// The docs the manifest gives.
    pub count: usize,
    /// The manifest's description, when it has one.
    pub description: Option<String>,
    /// The manifest's owner, when it has one.
    pub owner: Option<String>,
}

/// A log stream: its name, its entry count, how many of its entries are new, and the stamp of its newest entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stream {
    /// The stream's folder name.
    pub name: String,
    /// Every entry in the stream.
    pub count: usize,
    /// The entries no more than 24 hours old.
    pub new_24h: usize,
    /// The RFC 3339 time of the newest entry, when its name carries a stamp.
    pub newest: Option<String>,
}

/// The entries addressed to an agent: the agent's id and how many entries name it in their `to` list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToCount {
    /// The agent id, as the atlas was given it.
    pub agent: String,
    /// The entries in the store whose `to` list holds the agent.
    pub count: usize,
}

/// The nearest README of the cwd: its folder's label and its one-liner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereLine {
    /// The root-relative label of the folder the README is in, `.` for the root.
    pub path: String,
    /// The README's one-liner, in full.
    pub title: String,
}

/// The text of the atlas (design §3.5): the header, then one line per part that has something to show. Text over the cap has
/// its longest list line cut to its first items and a `+N more` ending, until the whole text is under the cap.
pub fn text(atlas: &Atlas) -> String {
    let mut lists = [
        List::new("collections", collection_items(&atlas.collections)),
        List::new("streams", stream_items(&atlas.streams)),
    ];
    loop {
        let out = assemble(atlas, &lists);
        if out.len() <= CAP {
            return out;
        }
        let longest = lists
            .iter_mut()
            .filter(|list| list.shown > 0)
            .max_by_key(|list| list.line_len());
        match longest {
            Some(list) => list.shown -= 1,
            None => return out,
        }
    }
}

/// The `--json` document (design §3.6): one object with the root, the store, the collections and the warnings, and the `to`
/// and `here` objects when they show, with one line break at the end.
pub fn document(atlas: &Atlas) -> String {
    let mut out = json!({
        "root": atlas.root,
        "log_store": atlas.store,
        "collections": collections_json(atlas),
        "warnings": atlas.warnings,
    });
    if let Some(to) = &atlas.to {
        out["to"] = json!({"agent": to.agent, "count": to.count});
    }
    if let Some(here) = &atlas.here {
        out["here"] = json!({"path": here.path, "title": here.title});
    }
    format!("{out}\n")
}

/// A list line of the text: its label, its items, and how many of them the cap lets it show.
struct List {
    /// The label printed before the value.
    label: &'static str,
    /// Every item of the list, in the order they print.
    items: Vec<String>,
    /// How many items print before the `+N more` ending; all of them when the list is not cut.
    shown: usize,
}

impl List {
    /// A list that shows every item.
    fn new(label: &'static str, items: Vec<String>) -> List {
        let shown = items.len();
        List {
            label,
            items,
            shown,
        }
    }

    /// The value of the line: the shown items joined with ` · `, then `+N more` when some items are cut.
    fn value(&self) -> String {
        let mut parts: Vec<String> = self.items[..self.shown].to_vec();
        if self.shown < self.items.len() {
            parts.push(format!("+{} more", self.items.len() - self.shown));
        }
        parts.join(DOT)
    }

    /// The length in bytes of the whole line, label included, which the cap compares.
    fn line_len(&self) -> usize {
        self.label.len() + self.value().len()
    }
}

/// The whole text with each list at its current length: the header, the intent line, each list that has items, the to and
/// here lines, then the use line. Every label is padded to the longest label that prints, and the value follows two spaces on.
fn assemble(atlas: &Atlas, lists: &[List]) -> String {
    let mut lines: Vec<(String, String)> =
        vec![("intent".to_string(), intent_value(&atlas.intent))];
    for list in lists.iter().filter(|list| !list.items.is_empty()) {
        lines.push((list.label.to_string(), list.value()));
    }
    if let Some(to) = &atlas.to {
        lines.push((format!("to {}", to.agent), to_value(to)));
    }
    if let Some(here) = &atlas.here {
        lines.push(("here".to_string(), here_value(here)));
    }
    lines.push(("use".to_string(), USE.to_string()));
    let width = lines
        .iter()
        .map(|(label, _)| label.len())
        .max()
        .unwrap_or(0);
    let mut out = format!("memex  {} · log store {}\n", atlas.root, atlas.store_label);
    for (label, value) in lines {
        out.push_str(&format!("{label:<width$}  {value}\n"));
    }
    out
}

/// The intent line's value: the built-in counts in the order readmes, headers and cases.
fn intent_value(intent: &Intent) -> String {
    [
        counted(intent.readmes, "readme", "readmes"),
        counted(intent.headers, "header", "headers"),
        counted(intent.cases, "case", "cases"),
    ]
    .join(DOT)
}

/// A count with its noun, in the singular for one and the plural otherwise.
fn counted(count: usize, one: &str, many: &str) -> String {
    let noun = if count == 1 { one } else { many };
    format!("{count} {noun}")
}

/// The items of the collections line: each name and its doc count.
fn collection_items(collections: &[Collection]) -> Vec<String> {
    collections
        .iter()
        .map(|collection| format!("{} {}", collection.name, collection.count))
        .collect()
}

/// The items of the streams line: each name and its entry count, then how many are new. The first stream with new entries says
/// `in 24h`, and the later ones say only `new`; a stream with none says nothing (design §3.5).
fn stream_items(streams: &[Stream]) -> Vec<String> {
    let mut items = Vec::new();
    let mut first = true;
    for stream in streams {
        let note = match stream.new_24h {
            0 => String::new(),
            count if first => {
                first = false;
                format!(" ({count} new in 24h)")
            }
            count => format!(" ({count} new)"),
        };
        items.push(format!("{} {}{note}", stream.name, stream.count));
    }
    items
}

/// The value of the to line: the count of messages, then the search that finds them.
fn to_value(to: &ToCount) -> String {
    let messages = counted(to.count, "message", "messages");
    format!("{messages} → memex search to:{}", to.agent)
}

/// The value of the here line: the folder, then its one-liner cut at 120 characters; the dash and the one-liner are left out
/// when the README has none.
fn here_value(here: &HereLine) -> String {
    if here.title.is_empty() {
        here.path.clone()
    } else {
        format!("{} — {}", here.path, clip(&here.title))
    }
}

/// The collections of the JSON document (design §3.6): the three built-ins, then the manifests, then the streams. A built-in has
/// no kind, description or owner; a stream has kind `log` and its new count, and its newest time when it has one.
fn collections_json(atlas: &Atlas) -> Vec<Json> {
    let intent = &atlas.intent;
    let mut list = vec![
        builtin(crate::corpus::READMES, intent.readmes),
        builtin(crate::corpus::HEADERS, intent.headers),
        builtin(crate::corpus::CASES, intent.cases),
    ];
    list.extend(atlas.collections.iter().map(|collection| {
        json!({
            "name": collection.name,
            "kind": collection.kind,
            "builtin": false,
            "count": collection.count,
            "description": collection.description,
            "owner": collection.owner,
        })
    }));
    list.extend(atlas.streams.iter().map(stream_json));
    list
}

/// A built-in collection as JSON: its name and its count, with no kind, description or owner.
fn builtin(name: &str, count: usize) -> Json {
    json!({
        "name": name,
        "kind": null,
        "builtin": true,
        "count": count,
        "description": null,
        "owner": null,
    })
}

/// A log stream as JSON: its kind is `log`, and it carries its new count and, when it has one, its newest time.
fn stream_json(stream: &Stream) -> Json {
    let mut item = json!({
        "name": stream.name,
        "kind": "log",
        "builtin": false,
        "count": stream.count,
        "description": null,
        "owner": null,
        "new_24h": stream.new_24h,
    });
    if let Some(newest) = &stream.newest {
        item["newest"] = json!(newest);
    }
    item
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_of_one_is_singular() {
        assert_eq!(counted(1, "readme", "readmes"), "1 readme");
        assert_eq!(counted(0, "readme", "readmes"), "0 readmes");
        assert_eq!(counted(2, "readme", "readmes"), "2 readmes");
    }

    /// An atlas whose root is `root` and whose streams line lists forty streams, so it is long enough to be cut.
    fn atlas_with_root(root: String) -> Atlas {
        let stream = |index| Stream {
            name: format!("stream-{index:02}-with-a-long-name"),
            count: 1,
            new_24h: 0,
            newest: None,
        };
        Atlas {
            root,
            store: None,
            store_label: "none (run memex init)".to_string(),
            intent: Intent::default(),
            collections: Vec::new(),
            streams: (0..40).map(stream).collect(),
            to: None,
            here: None,
            warnings: Vec::new(),
        }
    }

    /// INFO-CAP-02: a text of exactly 3,000 bytes is not cut, and one of 3,001 bytes is. The root is padded to hit the boundary,
    /// which no golden can do on every OS, since the sandbox root's length differs.
    #[test]
    fn info_cap_02_a_text_of_exactly_the_cap_is_not_cut_and_one_byte_more_is() {
        let short = text(&atlas_with_root(String::new()));
        assert!(short.len() < CAP && !short.contains("more"), "{short}");
        let exact = text(&atlas_with_root("/".repeat(CAP - short.len())));
        assert!(exact.len() == CAP && !exact.contains("more"), "{exact}");
        let over = text(&atlas_with_root("/".repeat(CAP - short.len() + 1)));
        assert!(over.len() <= CAP && over.contains("+1 more"), "{over}");
    }

    #[test]
    fn a_cut_list_ends_with_its_count_of_the_rest() {
        let mut list = List::new("streams", vec!["a 1".to_string(), "b 2".to_string()]);
        assert_eq!(list.value(), "a 1 · b 2");
        list.shown = 1;
        assert_eq!(list.value(), "a 1 · +1 more");
        list.shown = 0;
        assert_eq!(list.value(), "+2 more");
    }
}
