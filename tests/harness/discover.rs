//! Lists the families and case folders under a cases root, and parses each family's CASES.md.
//! Cases: all (runner)

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// One entry of a directory listing.
pub struct Item {
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
}

/// Lists a directory sorted by name. A symlink is neither a file nor a folder here.
pub fn list(dir: &Path) -> io::Result<Vec<Item>> {
    let mut items = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        items.push(Item {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_file: kind.is_file(),
            is_dir: kind.is_dir(),
        });
    }
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// One well-formed entry line of a CASES.md.
pub struct Entry {
    pub id: String,
    /// 1-based line number in CASES.md.
    pub line: usize,
    /// The text of the `## ` heading above the entry, if there is one.
    pub heading: Option<String>,
    /// True when the claim contains "(planned)": such an entry has no folder yet.
    pub planned: bool,
}

/// A parsed CASES.md: its entries, and the lines that start like an entry but are malformed.
pub struct CasesMd {
    pub entries: Vec<Entry>,
    /// Pairs of 1-based line number and the line itself.
    pub malformed: Vec<(usize, String)>,
}

/// Parses CASES.md. `## ` headings set the topic; a column-0 line that starts with "- `" must be an entry.
pub fn parse_cases_md(text: &str) -> CasesMd {
    let mut parsed = CasesMd {
        entries: Vec::new(),
        malformed: Vec::new(),
    };
    let mut heading: Option<String> = None;
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if let Some(title) = line.strip_prefix("## ") {
            heading = Some(title.trim().to_string());
        } else if line.starts_with("- `") {
            match parse_entry(line) {
                Some((id, claim)) => parsed.entries.push(Entry {
                    planned: claim.contains("(planned)"),
                    id,
                    line: number,
                    heading: heading.clone(),
                }),
                None => parsed.malformed.push((number, line.to_string())),
            }
        }
    }
    parsed
}

/// Matches the entry shape "- `ID` claim", where the claim has at least one character.
fn parse_entry(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("- `")?;
    let close = rest.find('`')?;
    let id = &rest[..close];
    if !is_id(id) {
        return None;
    }
    let claim = rest[close + 1..].strip_prefix(' ')?;
    if claim.is_empty() {
        return None;
    }
    Some((id.to_string(), claim.to_string()))
}

/// True for PREFIX-TOPIC-NN: capitals, a dash, capitals, a dash, and exactly two ASCII digits.
pub fn is_id(id: &str) -> bool {
    let parts: Vec<&str> = id.split('-').collect();
    match parts.as_slice() {
        [prefix, topic, number] => {
            is_capitals(prefix)
                && is_capitals(topic)
                && number.len() == 2
                && number.bytes().all(|b| b.is_ascii_digit())
        }
        _ => false,
    }
}

fn is_capitals(part: &str) -> bool {
    !part.is_empty() && part.bytes().all(|b| b.is_ascii_uppercase())
}

/// The family prefix of an id: its first segment.
pub fn prefix_of(id: &str) -> &str {
    id.split('-').next().unwrap_or("")
}

/// The topic of an id: its middle segment, which must name its `## ` heading.
pub fn topic_of(id: &str) -> &str {
    id.split('-').nth(1).unwrap_or("")
}

/// A case: a non-planned entry whose folder exists under its family.
pub struct CaseRef {
    pub id: String,
    pub dir: PathBuf,
}

/// Every case under `root`, sorted by id. Conformity runs first, so these lines up with CASES.md.
pub fn cases(root: &Path) -> Vec<CaseRef> {
    let mut found = Vec::new();
    let Ok(items) = list(root) else {
        return found;
    };
    for family in items.iter().filter(|item| item.is_dir) {
        let family_dir = root.join(&family.name);
        let Ok(text) = fs::read_to_string(family_dir.join("CASES.md")) else {
            continue;
        };
        for entry in parse_cases_md(&text)
            .entries
            .into_iter()
            .filter(|e| !e.planned)
        {
            let dir = family_dir.join(&entry.id);
            if dir.is_dir() {
                found.push(CaseRef { id: entry.id, dir });
            }
        }
    }
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}
