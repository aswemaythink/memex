//! Checks the six conformity rules of PLAN.md §2 over a cases tree, one line per violation; this file walks the tree and owns rules 1 and 6.
//! Cases: all (runner)

mod case_dir;
mod cases_md;
mod placeholders;

use std::path::Path;

use super::discover::{list, Entry, Item};

pub use placeholders::placeholder_problems;

const README: &str = "README.md";
const CASES: &str = "CASES.md";

/// Every violation of the six rules, one line each, in rule order.
///
/// Each line reads `ruleN: <what> at <path>: <how to fix>`, and the path is relative to `root`
/// with `/` separators. An empty list means the tree conforms.
pub fn check(root: &Path) -> Vec<String> {
    let mut report = Report::default();
    check_root(root, &mut report);
    report.lines.concat()
}

/// Violations grouped by rule: index 0 holds rule 1.
#[derive(Default)]
struct Report {
    lines: [Vec<String>; 6],
}

impl Report {
    fn add(&mut self, rule: usize, what: &str, at: &str, how: &str) {
        self.lines[rule - 1].push(format!("rule{rule}: {what} at {at}: {how}"));
    }
}

/// A family folder after rule 1 has looked at its layout.
struct Family {
    name: String,
    case_dirs: Vec<String>,
    /// The parsed CASES.md, or None when it is missing, unreadable or not UTF-8.
    entries: Option<Vec<Entry>>,
}

fn check_root(root: &Path, r: &mut Report) {
    let Some(family_names) = check_hub(root, r) else {
        return;
    };
    let families: Vec<Family> = family_names
        .iter()
        .filter_map(|name| check_family(root, name, r))
        .collect();
    cases_md::check_across_families(&families, r);
    for family in &families {
        cases_md::check_entries_against_folders(family, r);
        for case in &family.case_dirs {
            let dir = root.join(&family.name).join(case);
            case_dir::check_case(&dir, &family.name, case, r);
        }
    }
    if families.iter().all(|f| f.case_dirs.is_empty()) {
        r.add(
            6,
            "no case exists",
            ".",
            "add a family folder with at least one case folder; see tests/cases/README.md",
        );
    }
}

/// Rule 1 at the cases root: only the hub README.md and family folders. Returns the family names.
fn check_hub(root: &Path, r: &mut Report) -> Option<Vec<String>> {
    let items = match list(root) {
        Ok(items) => items,
        Err(e) => {
            r.add(
                1,
                &format!("cannot read the cases root ({:?})", e.kind()),
                ".",
                "create it with a README.md; each family is a folder inside it",
            );
            return None;
        }
    };
    let mut family_names = Vec::new();
    for item in &items {
        if item.is_dir {
            family_names.push(item.name.clone());
        } else if !(item.is_file && item.name == README) {
            r.add(
                1,
                "stray entry",
                &item.name,
                "only README.md and family folders belong at this level; move it into a family folder or delete it",
            );
        }
    }
    if !has_file(&items, README) {
        r.add(
            1,
            "hub README.md is missing",
            README,
            "add it; it describes the case convention",
        );
    }
    Some(family_names)
}

/// Rule 1 for one family folder, and rule 2 for its CASES.md.
fn check_family(root: &Path, name: &str, r: &mut Report) -> Option<Family> {
    let dir = root.join(name);
    let items = match list(&dir) {
        Ok(items) => items,
        Err(e) => {
            r.add(
                1,
                &format!("cannot read the family folder ({:?})", e.kind()),
                name,
                "check its permissions",
            );
            return None;
        }
    };
    if !has_file(&items, README) {
        r.add(
            1,
            &format!("family {name} has no README.md"),
            &format!("{name}/{README}"),
            "add it; its first paragraph says what the family covers",
        );
    }
    let has_cases = has_file(&items, CASES);
    if !has_cases {
        r.add(
            1,
            &format!("family {name} has no CASES.md"),
            &format!("{name}/{CASES}"),
            "add the claim index: a title, then one \"## TOPIC\" heading per topic with its entries",
        );
    }
    let case_dirs = case_dirs_of(&items, name, r);
    let entries = has_cases
        .then(|| cases_md::parse_family(&dir, name, r))
        .flatten();
    Some(Family {
        name: name.to_string(),
        case_dirs,
        entries,
    })
}

/// Rule 1 inside a family folder: its folders are the case folders, and no other file belongs.
fn case_dirs_of(items: &[Item], name: &str, r: &mut Report) -> Vec<String> {
    let mut case_dirs = Vec::new();
    for item in items {
        if item.is_dir {
            case_dirs.push(item.name.clone());
        } else if !(item.is_file && (item.name == README || item.name == CASES)) {
            r.add(
                1,
                "stray entry",
                &format!("{name}/{}", item.name),
                "a family folder holds README.md, CASES.md and case folders only; move the file into a case folder or delete it",
            );
        }
    }
    case_dirs
}

fn has_file(items: &[Item], name: &str) -> bool {
    items.iter().any(|i| i.is_file && i.name == name)
}
