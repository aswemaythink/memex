//! Runs the built memex on this repo and checks its intent layer (PLAN §3).
//! It checks the cases, the headers, the READMEs and the links.
//! Cases: none (a self-check of this repo, not a conformance case)

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

/// The trees of the repo that the header and README checks walk, relative to the crate root.
const ROOTS: [&str; 2] = ["src", "tests"];

/// The runner's deliberately broken trees, which are data and not part of the repo's intent layer.
const SELFTEST_TREE: &str = "tests/runner-selftest";

/// The family folders of the conformance cases. A folder two levels below it is a case folder, which is data.
const CASES_TREE: &str = "tests/cases/";

#[test]
fn every_case_is_cited_directly() {
    let roster = memex_json(&["search", "--in", "cases", "--limit", "100000"]);
    let mut uncited = Vec::new();
    for hit in hits_of(&roster) {
        let planned = hit["title"].as_str().unwrap_or("").contains("(planned)");
        if !planned && hit["cited_by"].as_u64() == Some(0) {
            uncited.push(hit["ref"].as_str().unwrap_or("?").to_string());
        }
    }
    report(
        &uncited,
        "cite each case directly: add its id to the `//! Cases:` line of the file that implements it, or name it in a README",
    );
}

#[test]
fn every_source_file_has_a_header() {
    let folders = gated_folders();
    let mut headerless = Vec::new();
    for doc in shown(&folders) {
        for item in doc["children"]["items"].as_array().into_iter().flatten() {
            let path = item["path"].as_str().unwrap_or("?");
            if item["kind"] == "file" && path.ends_with(".rs") && item["title"].is_null() {
                headerless.push(format!("{path}: no `//!` header"));
            }
        }
    }
    report(
        &headerless,
        "open the file with `//!` lines whose first sentence says what the file is for",
    );
}

#[test]
fn every_folder_has_a_readme() {
    let folders = gated_folders();
    let mut readmeless = Vec::new();
    for (folder, doc) in folders.iter().zip(shown(&folders)) {
        if doc["title"].is_null() {
            readmeless.push(format!("{folder}/: no README.md"));
        }
    }
    report(
        &readmeless,
        "add a README.md whose first paragraph says what the folder is",
    );
}

#[test]
fn no_doc_under_src_has_an_unresolved_citation() {
    let mut paths = Vec::new();
    all_under("src", &mut paths);
    let mut unresolved = Vec::new();
    for doc in shown(&paths) {
        for cite in doc["cites"].as_array().into_iter().flatten() {
            if cite["resolved"] == false {
                unresolved.push(format!(
                    "{}: cites {}, which resolves to nothing",
                    doc["ref"].as_str().unwrap_or("?"),
                    cite["ref"].as_str().unwrap_or("?")
                ));
            }
        }
    }
    report(
        &unresolved,
        "make each reference name a file, folder or doc that exists, or reword it so it is not a reference",
    );
}

/// AGENTS.md's marked block is assets/agents-snippet.md byte for byte: the one source that `init` writes (design §6, PLAN §4).
#[test]
fn agents_block_is_the_snippet_byte_for_byte() {
    let agents = read_repo("AGENTS.md");
    let snippet = read_repo("assets/agents-snippet.md");
    let block = agents
        .split_once("<!-- memex:begin -->\n")
        .and_then(|(_, rest)| rest.split_once("<!-- memex:end -->"))
        .map(|(block, _)| block);
    assert_eq!(
        block,
        Some(snippet.as_str()),
        "the marked block in AGENTS.md must equal assets/agents-snippet.md byte for byte; fix: copy the snippet between the markers, or run `memex init` in this repo"
    );
}

/// The text of a file under the crate root. A file that cannot be read fails the check.
fn read_repo(rel: &str) -> String {
    fs::read_to_string(manifest().join(rel)).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// Runs the built memex with `args` and `--json` on this repo, with MEMEX_NOW unset, and returns its stdout as JSON. Exit codes
/// are not judged here: a failed ref is reported by the JSON `errors`, which each check reads.
fn memex_json(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_memex"))
        .args(args)
        .arg("--json")
        .current_dir(manifest())
        .env_remove("MEMEX_NOW")
        .output()
        .unwrap_or_else(|e| panic!("cannot run memex: {e}"));
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "memex printed no JSON for {args:?}: {e}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

/// The docs that `memex show --json` gives for the paths, one per path in order, read with every child listed. A ref that
/// memex cannot read fails the check, since its doc would be missing.
fn shown(paths: &[String]) -> Vec<Value> {
    let mut args: Vec<&str> = vec!["show"];
    args.extend(paths.iter().map(String::as_str));
    args.extend(["--limit", "100000"]);
    let json = memex_json(&args);
    let errors = json["errors"].as_array().map_or(0, Vec::len);
    assert_eq!(errors, 0, "memex show could not read every path: {json}");
    let docs: Vec<Value> = json["docs"].as_array().cloned().unwrap_or_default();
    assert_eq!(
        docs.len(),
        paths.len(),
        "memex show did not give one doc per path"
    );
    docs
}

/// The hits of a search document: every hit of every group, in order.
fn hits_of(document: &Value) -> Vec<&Value> {
    document["groups"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|group| group["hits"].as_array().into_iter().flatten())
        .collect()
}

/// The folders that the checks gate, under the roots: each folder that is not a fixture, not in the self-test trees and not a
/// case folder. Paths are `/`-separated and relative to the crate root.
fn gated_folders() -> Vec<String> {
    let mut folders = Vec::new();
    for root in ROOTS {
        collect_gated(root, &mut folders);
    }
    folders
}

/// Adds `dir` and every gated folder below it.
fn collect_gated(dir: &str, folders: &mut Vec<String>) {
    folders.push(dir.to_string());
    for (child, is_dir) in entries(dir) {
        if is_dir && gated(&child) {
            collect_gated(&child, folders);
        }
    }
}

/// Adds `path`, every folder and file below it, with each folder before its children.
fn all_under(path: &str, out: &mut Vec<String>) {
    out.push(path.to_string());
    for (child, is_dir) in entries(path) {
        if is_dir {
            all_under(&child, out);
        } else {
            out.push(child);
        }
    }
}

/// True when a folder is gated: not a fixture tree, not under the runner's self-test trees, and not a case folder.
fn gated(rel: &str) -> bool {
    let fixture = rel.split('/').any(|part| part == "fixture");
    !fixture && !rel.starts_with(SELFTEST_TREE) && !is_case_folder(rel)
}

/// True for a case folder, `tests/cases/<family>/<ID>`, and for anything below one.
fn is_case_folder(rel: &str) -> bool {
    rel.strip_prefix(CASES_TREE)
        .is_some_and(|rest| rest.split('/').count() >= 2)
}

/// The children of a folder, each as its `/`-separated path from the crate root and whether it is a folder, sorted by path.
fn entries(dir: &str) -> Vec<(String, bool)> {
    let full = manifest().join(dir);
    let listing =
        fs::read_dir(&full).unwrap_or_else(|e| panic!("cannot list {}: {e}", full.display()));
    let mut found = Vec::new();
    for entry in listing {
        let entry = entry.unwrap_or_else(|e| panic!("cannot read {}: {e}", full.display()));
        let name = entry.file_name().to_string_lossy().into_owned();
        found.push((format!("{dir}/{name}"), entry.path().is_dir()));
    }
    found.sort();
    found
}

/// The crate root: the directory of this package's manifest.
fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Passes when there are no problems. Otherwise fails, naming every problem and the fix.
fn report(problems: &[String], fix: &str) {
    assert!(
        problems.is_empty(),
        "{} problem(s):\n  {}\nfix: {fix}",
        problems.len(),
        problems.join("\n  ")
    );
}
