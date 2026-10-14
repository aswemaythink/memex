//! Prints `show`'s sections and misses as text (design §3.3).
//! Each section sits under its `===` line, then its blocks in order, with a limits line after the children and cited-by lists.
//! Cases: SHOW-FILE-01, SHOW-FILE-02, SHOW-DIR-01, SHOW-DIR-02, SHOW-MD-01, SHOW-MD-02, SHOW-LOG-01, SHOW-UP-01, SHOW-ROOT-01, SHOW-LIMIT-01, SHOW-LIMIT-02, SHOW-SEP-01, SHOW-ERR-01, SHOW-ERR-02, SHOW-LINE-01, INT-NOHEAD-03, CASE-SECT-01, CITE-PREFIX-01, CITE-UNRES-01, CITE-BACK-01, CITE-BACK-04, CITE-LINK-03, SHOW-OTHER-01

use super::{CaseParts, Citations, Fields, Link, Listing, Miss, Ref, Section};
use crate::cite::Cite;
use crate::collections::frontmatter::Value;
use crate::corpus::{Child, Kind};

/// A one-liner is cut to this many characters, the ellipsis included (design §2).
const CLIP: usize = 120;

/// The text of several sections: each ends in a newline, and one blank line separates two sections.
pub fn sections(sections: &[Section]) -> String {
    let parts: Vec<String> = sections.iter().map(section).collect();
    parts.join("\n")
}

/// The stderr text of the misses: `memex: <message>`, then one indented line per candidate.
pub fn misses(misses: &[Miss]) -> String {
    let mut out = String::new();
    for miss in misses {
        out.push_str(&format!("memex: {}\n", miss.message));
        for candidate in &miss.candidates {
            out.push_str(&format!("  {candidate}\n"));
        }
    }
    out
}

/// One section: its `===` line, its body, then the blocks it has, in the order of design §3.3.
fn section(section: &Section) -> String {
    let mut out = format!("=== {} ===\n", section.label);
    body(&mut out, section);
    if let Some(Fields::Case(case)) = &section.fields {
        // A case has no cites block: design §3.3 lists its claim, detail, group, defining CASES.md, evidence and cited by.
        case_blocks(&mut out, section.path.as_deref(), case);
        cited_by_block(&mut out, &section.citations);
        return out;
    }
    if let Some(Fields::Pairs(pairs)) = &section.fields {
        fields_block(&mut out, pairs);
    }
    if let Some(children) = &section.children {
        children_block(&mut out, children);
    }
    cites_block(&mut out, &section.citations);
    cited_by_block(&mut out, &section.citations);
    if let Some(chain) = &section.chain {
        chain_block(&mut out, chain);
    }
    out
}

/// The body: the text in full, or the placeholder when there is no text.
fn body(out: &mut String, section: &Section) {
    match &section.text {
        Some(text) => out.push_str(&with_newline(text)),
        None => out.push_str(&format!("{}\n", section.missing)),
    }
}

/// The fields block: `key: value` lines, with lists written `[a, b]`.
fn fields_block(out: &mut String, fields: &[(String, Value)]) {
    start(out, "fields");
    if fields.is_empty() {
        out.push_str("  (none)\n");
    }
    for (key, value) in fields {
        out.push_str(&format!("  {key}: {}\n", value_text(value)));
    }
}

/// The blocks of a case after its claim and detail: its group, the CASES.md that defines it, and its evidence. Each shows
/// `(none)` when it has nothing.
fn case_blocks(out: &mut String, defined_in: Option<&str>, case: &CaseParts) {
    start(out, "group");
    out.push_str(&format!(
        "  {}\n",
        case.group.as_deref().unwrap_or("(none)")
    ));
    start(out, "defined in");
    out.push_str(&format!("  {}\n", defined_in.unwrap_or("(none)")));
    start(out, "evidence");
    if case.evidence.is_empty() {
        out.push_str("  (none)\n");
    }
    for path in &case.evidence {
        out.push_str(&format!("  {path}\n"));
    }
}

/// The children block, one line per child, then the limits line.
fn children_block(out: &mut String, children: &Listing<Child>) {
    start(out, "children");
    if children.total == 0 {
        out.push_str("  (none)\n");
    }
    for child in &children.items {
        out.push_str(&child_line(child));
    }
    limits(out, children.items.len(), children.total);
}

/// The cites block: one line per citation, or `(none)`. An entry of another root names its root in place of the list.
fn cites_block(out: &mut String, citations: &Citations) {
    start(out, "cites");
    match citations {
        Citations::Listed { cites, .. } => cite_lines(out, cites),
        Citations::OtherRoot { root } => out.push_str(&other_root_line(root)),
    }
}

/// One line per citation, or `(none)` when there is none.
fn cite_lines(out: &mut String, cites: &[Cite]) {
    if cites.is_empty() {
        out.push_str("  (none)\n");
    }
    for cite in cites {
        if cite.resolved {
            out.push_str(&format!(
                "  {}{}\n",
                cite.reference,
                title_part(&cite.title)
            ));
        } else {
            out.push_str(&format!("  {}  unresolved\n", cite.reference));
        }
    }
}

/// The cited-by block: one line per citing ref, marked with the prefix it cites by, then the limits line. An entry of another
/// root names its root in place of the list, with no limits line.
fn cited_by_block(out: &mut String, citations: &Citations) {
    start(out, "cited by");
    match citations {
        Citations::Listed { cited_by, .. } => cited_by_lines(out, cited_by),
        Citations::OtherRoot { root } => out.push_str(&other_root_line(root)),
    }
}

/// One line per citing ref, or `(none)` when there is none, then the limits line.
fn cited_by_lines(out: &mut String, cited: &Listing<Ref>) {
    if cited.total == 0 {
        out.push_str("  (none)\n");
    }
    for item in &cited.items {
        let title = item.title.as_deref().unwrap_or("");
        let via = item
            .via
            .as_deref()
            .map_or_else(String::new, |prefix| format!(" (via {prefix})"));
        out.push_str(&format!("  {}{}{via}\n", item.label, title_part(title)));
    }
    limits(out, cited.items.len(), cited.total);
}

/// The line that stands in for a list of an entry in another root: the root that holds the list (design §3.1).
fn other_root_line(root: &str) -> String {
    format!("  (in another root: run show from {root})\n")
}

/// The chain block: each README with its `---` line, nearest first, or `(none)`.
fn chain_block(out: &mut String, chain: &[Link]) {
    start(out, "chain");
    if chain.is_empty() {
        out.push_str("  (none)\n");
    }
    for link in chain {
        out.push_str(&format!("--- {}\n", link.path));
        out.push_str(&with_newline(&link.text));
    }
}

/// The line of one child: its name, with `/` for a folder, then its one-liner. A folder whose README exists but has no
/// one-liner shows its name alone; only a folder with no README, or a file with no header, shows a placeholder.
fn child_line(child: &Child) -> String {
    let (name, placeholder) = match child.kind {
        Kind::Dir => (format!("{}/", child.name), "(no README)"),
        Kind::File => (child.name.clone(), "(no header)"),
    };
    match (&child.title, child.kind) {
        (Some(title), Kind::Dir) if title.is_empty() => format!("  {name}\n"),
        (Some(title), _) if !title.is_empty() => format!("  {name}  {}\n", clip(title)),
        _ => format!("  {name}  {placeholder}\n"),
    }
}

/// The limits line of a list: how many rows it shows, and the total. The flag that shows them all is named only when the
/// list was cut, so `shown` is less than `total` (design §3.3 Limits).
fn limits(out: &mut String, shown: usize, total: usize) {
    out.push_str(&format!("  showing {shown} of {total}"));
    if shown < total {
        out.push_str(&format!(" (--limit {total} for all)"));
    }
    out.push('\n');
}

/// A blank line and a block's heading, which sits alone on its line.
fn start(out: &mut String, heading: &str) {
    out.push_str(&format!("\n{heading}\n"));
}

/// Two spaces and a one-liner, clipped; nothing when there is no one-liner.
fn title_part(title: &str) -> String {
    if title.is_empty() {
        String::new()
    } else {
        format!("  {}", clip(title))
    }
}

/// A field value as text: a scalar as written, a list as `[a, b]`, with nested lines indented under their key.
fn value_text(value: &Value) -> String {
    match value {
        Value::Str(text) => text.replace('\n', "\n    "),
        Value::List(items) => format!("[{}]", items.join(", ")),
    }
}

/// The text with a final newline: added when it is missing, and nothing added to an empty text.
fn with_newline(text: &str) -> String {
    if text.is_empty() || text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    }
}

/// A one-liner cut to `CLIP` characters: the first `CLIP - 1` and an ellipsis, when it is longer.
pub fn clip(text: &str) -> String {
    if text.chars().count() <= CLIP {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(CLIP - 1).collect();
    cut.push('…');
    cut
}
