//! Checks the six conformity rules of PLAN.md §2 over a cases tree, one line per violation.
//! Cases: all (runner)

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::discover::{list, parse_cases_md, prefix_of, topic_of, Entry};
use super::{load_spec, CaseSpec};

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
    let items = match list(root) {
        Ok(items) => items,
        Err(e) => {
            r.add(
                1,
                &format!("cannot read the cases root ({:?})", e.kind()),
                ".",
                "create it with a README.md; each family is a folder inside it",
            );
            return;
        }
    };
    let mut family_names = Vec::new();
    let mut hub_readme = false;
    for item in &items {
        if item.is_dir {
            family_names.push(item.name.clone());
        } else if item.is_file && item.name == README {
            hub_readme = true;
        } else {
            r.add(
                1,
                "stray entry",
                &item.name,
                "only README.md and family folders belong at this level; move it into a family folder or delete it",
            );
        }
    }
    if !hub_readme {
        r.add(
            1,
            "hub README.md is missing",
            README,
            "add it; it describes the case convention",
        );
    }

    let mut families = Vec::new();
    for name in &family_names {
        if let Some(family) = check_family(root, name, r) {
            families.push(family);
        }
    }
    check_across_families(&families, r);
    for family in &families {
        check_entries_against_folders(family, r);
        for case in &family.case_dirs {
            let dir = root.join(&family.name).join(case);
            check_case(&dir, &family.name, case, r);
        }
    }
    let cases: usize = families.iter().map(|f| f.case_dirs.len()).sum();
    if cases == 0 {
        r.add(
            6,
            "no case exists",
            ".",
            "add a family folder with at least one case folder; see tests/cases/README.md",
        );
    }
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
    let has_file = |file: &str| items.iter().any(|i| i.is_file && i.name == file);
    if !has_file(README) {
        r.add(
            1,
            &format!("family {name} has no README.md"),
            &format!("{name}/{README}"),
            "add it; its first paragraph says what the family covers",
        );
    }
    let has_cases = has_file(CASES);
    if !has_cases {
        r.add(
            1,
            &format!("family {name} has no CASES.md"),
            &format!("{name}/{CASES}"),
            "add the claim index: a title, then one \"## TOPIC\" heading per topic with its entries",
        );
    }
    let mut case_dirs = Vec::new();
    for item in &items {
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
    let entries = if has_cases {
        parse_family(&dir, name, r)
    } else {
        None
    };
    Some(Family {
        name: name.to_string(),
        case_dirs,
        entries,
    })
}

/// Rule 2 for one CASES.md: it must be UTF-8, its entries must parse, and each sits under the `## ` heading of its topic.
fn parse_family(dir: &Path, name: &str, r: &mut Report) -> Option<Vec<Entry>> {
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
    for entry in &parsed.entries {
        let topic = topic_of(&entry.id);
        match entry.heading.as_deref() {
            None => r.add(
                2,
                &format!(
                    "entry {} on line {} is above every \"## \" heading",
                    entry.id, entry.line
                ),
                &at,
                &format!("put it under a \"## {topic}\" heading"),
            ),
            Some(heading) if heading != topic => r.add(
                2,
                &format!(
                    "entry {} on line {} sits under \"## {heading}\" but its topic is {topic}",
                    entry.id, entry.line
                ),
                &at,
                &format!("move it under \"## {topic}\" or fix the id"),
            ),
            Some(_) => {}
        }
    }
    let mut family_prefix: Option<&str> = None;
    for entry in &parsed.entries {
        let prefix = prefix_of(&entry.id);
        match family_prefix {
            None => family_prefix = Some(prefix),
            Some(first) if first != prefix => r.add(
                2,
                &format!(
                    "entry {} on line {} has prefix {prefix} but family {name} uses {first}",
                    entry.id, entry.line
                ),
                &at,
                "give each family one prefix: move the entry to its own family or fix the id",
            ),
            Some(_) => {}
        }
    }
    Some(parsed.entries)
}

/// Rule 2 across families: each prefix belongs to one family, and each id is declared once.
fn check_across_families(families: &[Family], r: &mut Report) {
    let mut owner_of_prefix: BTreeMap<&str, &str> = BTreeMap::new();
    let mut first_declared: BTreeMap<&str, (&str, usize)> = BTreeMap::new();
    for family in families {
        let Some(entries) = &family.entries else {
            continue;
        };
        let at = format!("{}/{CASES}", family.name);
        if let Some(first) = entries.first() {
            let prefix = prefix_of(&first.id);
            match owner_of_prefix.get(prefix) {
                Some(owner) if *owner != family.name.as_str() => r.add(
                    2,
                    &format!("prefix {prefix} is also used by family {owner}"),
                    &at,
                    "give each family its own prefix",
                ),
                Some(_) => {}
                None => {
                    owner_of_prefix.insert(prefix, family.name.as_str());
                }
            }
        }
        for entry in entries {
            match first_declared.get(entry.id.as_str()) {
                Some((first_family, first_line)) => r.add(
                    2,
                    &format!(
                        "id {} is declared again on line {} (first on line {first_line} of {first_family})",
                        entry.id, entry.line
                    ),
                    &at,
                    "ids are unique repo-wide; delete the duplicate entry",
                ),
                None => {
                    first_declared.insert(entry.id.as_str(), (family.name.as_str(), entry.line));
                }
            }
        }
    }
}

/// Rule 3: every case folder has exactly one entry, and every entry has a folder unless it is planned.
fn check_entries_against_folders(family: &Family, r: &mut Report) {
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

/// Rules 4 and 5 for one case folder: its files, its case.json and its golden.
fn check_case(dir: &Path, family: &str, id: &str, r: &mut Report) {
    let at = |name: &str| format!("{family}/{id}/{name}");
    let items = match list(dir) {
        Ok(items) => items,
        Err(e) => {
            r.add(
                4,
                &format!("cannot read the case folder ({:?})", e.kind()),
                &format!("{family}/{id}"),
                "check its permissions",
            );
            return;
        }
    };
    let mut stdin_names: Vec<String> = Vec::new();
    match items.iter().find(|i| i.name == "case.json") {
        None => r.add(
            4,
            "case.json is missing",
            &at("case.json"),
            "add it; every case needs one (schema in tests/cases/README.md)",
        ),
        Some(item) if item.is_file => match load_spec(&dir.join("case.json")) {
            Err(err) => r.add(
                4,
                &format!("case.json does not parse against the schema ({err})"),
                &at("case.json"),
                "fix it; the schema is in tests/cases/README.md",
            ),
            Ok(spec) => check_spec(&spec, &at("case.json"), r, &mut stdin_names),
        },
        // A folder named case.json is reported as a stray entry below.
        Some(_) => {}
    }
    for item in &items {
        let allowed = match item.name.as_str() {
            "case.json" | "expect.txt" => item.is_file,
            "fixture" => item.is_dir,
            name => item.is_file && stdin_names.iter().any(|s| s == name),
        };
        if !allowed {
            r.add(
                4,
                "stray entry",
                &at(&item.name),
                "a case folder holds only case.json, expect.txt, fixture/ and named stdin files; delete it, or name it as a stdin_file",
            );
        }
    }
    for name in &stdin_names {
        if !items.iter().any(|i| i.is_file && i.name == *name) {
            r.add(
                4,
                &format!("stdin_file {name} is missing"),
                &at(name),
                "add the file, or fix the name in case.json",
            );
        }
    }
    if items.iter().any(|i| i.is_file && i.name == "expect.txt") {
        match fs::read(dir.join("expect.txt")) {
            Ok(bytes) => {
                for (token, line) in placeholder_problems(&bytes) {
                    r.add(
                        5,
                        &format!("placeholder {token:?} on line {line}"),
                        &at("expect.txt"),
                        "goldens may use only $SANDBOX and $VERSION; change the case so its output has no other $NAME token",
                    );
                }
            }
            Err(e) => r.add(
                5,
                &format!("cannot read expect.txt ({:?})", e.kind()),
                &at("expect.txt"),
                "check its permissions",
            ),
        }
    }
}

/// The schema checks that serde cannot make: steps, stdin, setup argv and platforms.
/// Also collects the plain stdin_file names, which the folder may hold.
fn check_spec(spec: &CaseSpec, at: &str, r: &mut Report, stdin_names: &mut Vec<String>) {
    if spec.steps.is_empty() {
        r.add(4, "case.json has no steps", at, "add at least one step");
    }
    for (index, step) in spec.steps.iter().enumerate() {
        let number = index + 1;
        if step.stdin.is_some() && step.stdin_file.is_some() {
            r.add(
                4,
                &format!("step {number} sets both stdin and stdin_file"),
                at,
                "keep one of them",
            );
        }
        if let Some(name) = &step.stdin_file {
            if is_plain_name(name) {
                stdin_names.push(name.clone());
            } else {
                r.add(
                    4,
                    &format!(
                        "step {number} names stdin_file {name:?}, which is not a plain file name"
                    ),
                    at,
                    "name a file directly inside the case folder",
                );
            }
        }
    }
    for (index, setup) in spec.setup.iter().enumerate() {
        let number = index + 1;
        match setup.run.first() {
            None => r.add(
                4,
                &format!("setup step {number} has an empty run list"),
                at,
                "give it an argv that starts with git or memex",
            ),
            Some(program) if program != "git" && program != "memex" => r.add(
                4,
                &format!("setup step {number} runs {program:?}"),
                at,
                "setup may run only git or memex",
            ),
            Some(_) => {}
        }
    }
    if spec.platforms.as_ref().is_some_and(|list| list.is_empty()) {
        r.add(
            4,
            "platforms is empty",
            at,
            "omit platforms to run on every OS, or list unix and/or windows",
        );
    }
}

/// A stdin_file must be one file name: no separators, and not `.` or `..`.
fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\'])
}

/// Every `$NAME` token in a golden other than `$SANDBOX` and `$VERSION`, with its 1-based line.
///
/// A token is `$` followed by `[A-Z_]`, then the run of `[A-Za-z0-9_]`. The `$ ` prompt of a
/// command line is not a token.
pub fn placeholder_problems(bytes: &[u8]) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    for (index, line) in bytes.split(|b| *b == b'\n').enumerate() {
        let mut at = 0;
        while at < line.len() {
            let starts_token = line[at] == b'$'
                && line
                    .get(at + 1)
                    .is_some_and(|b| b.is_ascii_uppercase() || *b == b'_');
            if !starts_token {
                at += 1;
                continue;
            }
            let mut end = at + 1;
            while end < line.len() && (line[end].is_ascii_alphanumeric() || line[end] == b'_') {
                end += 1;
            }
            let token = String::from_utf8_lossy(&line[at..end]).into_owned();
            if token != "$SANDBOX" && token != "$VERSION" {
                found.push((token, index + 1));
            }
            at = end;
        }
    }
    found
}
