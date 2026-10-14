//! Rules 4 and 5 of PLAN.md §2 for one case folder.
//! Only the allowed files, a case.json valid against the schema, and a golden with no stray placeholder.
//! Cases: all (runner)

use std::fs;
use std::path::Path;

use super::super::discover::{list, Item};
use super::super::{load_spec, CaseSpec};
use super::{has_file, placeholder_problems, Report};

/// Rules 4 and 5 for one case folder: its files, its case.json and its golden.
pub(super) fn check_case(dir: &Path, family: &str, id: &str, r: &mut Report) {
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
    let stdin_names = check_case_json(dir, &items, &at("case.json"), r);
    check_folder_items(&items, &stdin_names, &at, r);
    if has_file(&items, "expect.txt") {
        check_golden(&dir.join("expect.txt"), &at("expect.txt"), r);
    }
}

/// Rule 4 for case.json: it exists and parses. Returns the stdin_file names it allows in the folder.
fn check_case_json(dir: &Path, items: &[Item], at: &str, r: &mut Report) -> Vec<String> {
    let mut stdin_names = Vec::new();
    match items.iter().find(|i| i.name == "case.json") {
        None => r.add(
            4,
            "case.json is missing",
            at,
            "add it; every case needs one (schema in tests/cases/README.md)",
        ),
        // A folder named case.json is reported as a stray entry.
        Some(item) if !item.is_file => {}
        Some(_) => match load_spec(&dir.join("case.json")) {
            Err(err) => r.add(
                4,
                &format!("case.json does not parse against the schema ({err})"),
                at,
                "fix it; the schema is in tests/cases/README.md",
            ),
            Ok(spec) => check_spec(&spec, at, r, &mut stdin_names),
        },
    }
    stdin_names
}

/// Rule 4 for the folder: nothing but case.json, expect.txt, fixture/ and the named stdin files, all present.
fn check_folder_items(
    items: &[Item],
    stdin_names: &[String],
    at: &dyn Fn(&str) -> String,
    r: &mut Report,
) {
    for item in items {
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
    for name in stdin_names {
        if !has_file(items, name) {
            r.add(
                4,
                &format!("stdin_file {name} is missing"),
                &at(name),
                "add the file, or fix the name in case.json",
            );
        }
    }
}

/// Rule 5: the golden holds no placeholder but `$SANDBOX` and `$VERSION`.
fn check_golden(path: &Path, at: &str, r: &mut Report) {
    match fs::read(path) {
        Ok(bytes) => {
            for (token, line) in placeholder_problems(&bytes) {
                r.add(
                    5,
                    &format!("placeholder {token:?} on line {line}"),
                    at,
                    "goldens may use only $SANDBOX and $VERSION; change the case so its output has no other $NAME token",
                );
            }
        }
        Err(e) => r.add(
            5,
            &format!("cannot read expect.txt ({:?})", e.kind()),
            at,
            "check its permissions",
        ),
    }
}

/// The schema checks that serde cannot make: steps, stdin, paths, setup argv and platforms.
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
        if let Some(cwd) = &step.cwd {
            check_inside(&format!("step {number} cwd {cwd:?}"), cwd, at, r);
        }
    }
    for path in &spec.snapshot {
        check_inside(&format!("snapshot {path:?}"), path, at, r);
    }
    check_setup(spec, at, r);
    if spec.platforms.as_ref().is_some_and(|list| list.is_empty()) {
        r.add(
            4,
            "platforms is empty",
            at,
            "omit platforms to run on every OS, or list unix and/or windows",
        );
    }
}

/// Every setup argv starts with git or memex.
fn check_setup(spec: &CaseSpec, at: &str, r: &mut Report) {
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
}

/// A cwd or snapshot path must stay inside the sandbox, so no run can touch or show a path outside it.
fn check_inside(what: &str, path: &str, at: &str, r: &mut Report) {
    if !stays_inside(path) {
        r.add(
            4,
            &format!("{what} is not a path inside the sandbox"),
            at,
            "write it relative to the sandbox, such as repo/sub, with no .., no leading / and no drive",
        );
    }
}

/// True for a non-empty relative path with no `..` part, no leading separator and no drive or `:`.
fn stays_inside(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with(['/', '\\'])
        && !path.contains(':')
        && path.split(['/', '\\']).all(|part| part != "..")
}

/// A stdin_file must be one file name: no separators, and not `.` or `..`.
fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\'])
}
