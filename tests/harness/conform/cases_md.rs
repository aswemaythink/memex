//! Rules 2 and 3: each CASES.md parses, and its entries match the case folders.
//! Each CASES.md parses with one prefix, ids and prefixes are unique, and entries match case folders one to one.
//! Cases: all (runner)

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::super::discover::{parse_cases_md, prefix_of, topic_of, Entry};
use super::{Family, Report, CASES};

/// Rule 2 for one CASES.md: it must be UTF-8, its entries must parse, and each sits under the `## ` heading of its topic.
pub(super) fn parse_family(dir: &Path, name: &str, r: &mut Report) -> Option<Vec<Entry>> {
    let at = format!("{name}/{CASES}");
    let bytes = match fs::read(dir.join(CASES)) {
        Ok(bytes) => bytes,
        Err(e) => {
            r.add(
                2,
                &format!("cannot read CASES.md ({:?})", e.kind()),
                &at,
                "check its permissions",
            );
            return None;
        }
    };
    let Ok(text) = String::from_utf8(bytes) else {
        r.add(2, "CASES.md is not UTF-8", &at, "save it as UTF-8 text");
        return None;
    };
    let parsed = parse_cases_md(&text);
    for (line, content) in &parsed.malformed {
        r.add(
            2,
            &format!("malformed entry on line {line} ({content:?})"),
            &at,
            "write each entry as \"- `PREFIX-TOPIC-NN` claim\", with capitals and two digits, e.g. CLI-VERSION-01",
        );
    }
    check_headings(&parsed.entries, &at, r);
    check_one_prefix(&parsed.entries, name, &at, r);
    Some(parsed.entries)
}

/// Each entry sits under the `## ` heading that names its topic.
fn check_headings(entries: &[Entry], at: &str, r: &mut Report) {
    for entry in entries {
        let topic = topic_of(&entry.id);
        match entry.heading.as_deref() {
            None => r.add(
                2,
                &format!(
                    "entry {} on line {} is above every \"## \" heading",
                    entry.id, entry.line
                ),
                at,
                &format!("put it under a \"## {topic}\" heading"),
            ),
            Some(heading) if heading != topic => r.add(
                2,
                &format!(
                    "entry {} on line {} sits under \"## {heading}\" but its topic is {topic}",
                    entry.id, entry.line
                ),
                at,
                &format!("move it under \"## {topic}\" or fix the id"),
            ),
            Some(_) => {}
        }
    }
}

/// Every entry of a family shares the prefix of its first entry.
fn check_one_prefix(entries: &[Entry], name: &str, at: &str, r: &mut Report) {
    let Some(first) = entries.first().map(|e| prefix_of(&e.id)) else {
        return;
    };
    for entry in entries {
        let prefix = prefix_of(&entry.id);
        if prefix != first {
            r.add(
                2,
                &format!(
                    "entry {} on line {} has prefix {prefix} but family {name} uses {first}",
                    entry.id, entry.line
                ),
                at,
                "give each family one prefix: move the entry to its own family or fix the id",
            );
        }
    }
}

/// Rule 2 across families: each prefix belongs to one family, and each id is declared once.
pub(super) fn check_across_families(families: &[Family], r: &mut Report) {
    let mut owner_of_prefix: BTreeMap<&str, &str> = BTreeMap::new();
    let mut first_declared: BTreeMap<&str, (&str, usize)> = BTreeMap::new();
    for family in families {
        let Some(entries) = &family.entries else {
            continue;
        };
        let at = format!("{}/{CASES}", family.name);
        if let Some(first) = entries.first() {
            let prefix = prefix_of(&first.id);
            let owner = *owner_of_prefix.entry(prefix).or_insert(&family.name);
            if owner != family.name {
                r.add(
                    2,
                    &format!("prefix {prefix} is also used by family {owner}"),
                    &at,
                    "give each family its own prefix",
                );
            }
        }
        for entry in entries {
            if let Some((first_family, first_line)) = first_declared.get(entry.id.as_str()) {
                r.add(
                    2,
                    &format!(
                        "id {} is declared again on line {} (first on line {first_line} of {first_family})",
                        entry.id, entry.line
                    ),
                    &at,
                    "ids are unique repo-wide; delete the duplicate entry",
                );
            } else {
                first_declared.insert(&entry.id, (&family.name, entry.line));
            }
        }
    }
}

/// Rule 3: every case folder has exactly one entry, and every entry has a folder unless it is planned.
pub(super) fn check_entries_against_folders(family: &Family, r: &mut Report) {
    let Some(entries) = &family.entries else {
        return;
    };
    for case in &family.case_dirs {
        let count = entries.iter().filter(|e| e.id == *case).count();
        let at = format!("{}/{case}", family.name);
        if count == 0 {
            r.add(
                3,
                &format!("case dir {case} has no entry in CASES.md"),
                &at,
                &format!(
                    "add its entry to {}/{CASES}, or delete the folder",
                    family.name
                ),
            );
        } else if count > 1 {
            r.add(
                3,
                &format!("case dir {case} has {count} entries in CASES.md"),
                &at,
                "keep exactly one entry",
            );
        }
    }
    let mut reported = BTreeSet::new();
    for entry in entries {
        let has_dir = family.case_dirs.contains(&entry.id);
        let at = format!("{}/{}", family.name, entry.id);
        if entry.planned && has_dir {
            r.add(
                3,
                &format!("planned entry {} has a case dir", entry.id),
                &at,
                "delete the folder, or remove (planned) from the claim",
            );
        } else if !entry.planned && !has_dir && reported.insert(entry.id.as_str()) {
            r.add(
                3,
                &format!("entry {} has no case dir", entry.id),
                &at,
                "create the folder with case.json and expect.txt, or mark the claim (planned)",
            );
        }
    }
}
