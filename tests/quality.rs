//! Repo quality gate: source files stay short and open with a purpose header, every folder has a README.md, and no lint is silenced under src/ (tests/ may allow only dead_code).
//! Cases: none (repo quality gate)

use std::fs;
use std::path::{Path, PathBuf};

/// The most lines a source file may have.
const MAX_LINES: usize = 365;

/// The trees the gate walks, relative to the crate root.
const ROOTS: [&str; 2] = ["src", "tests"];

/// The only lint attribute tests/ may use.
const TEST_ALLOW: &str = "#[allow(dead_code)]";

/// One file or folder under a root.
struct Node {
    /// The path from the crate root, with `/` on every OS.
    rel: String,
    /// The path itself, for reading.
    path: PathBuf,
    /// True for a folder.
    is_dir: bool,
}

#[test]
fn the_gate_sees_the_source_tree() {
    let nodes = tree();
    let files = rust_files(&nodes);
    for must in ["src/lib.rs", "tests/quality.rs"] {
        assert!(
            files.iter().any(|node| node.rel == must),
            "the gate does not see {must}, so its walk is broken"
        );
    }
}

#[test]
fn source_files_are_at_most_365_lines() {
    let nodes = tree();
    let mut problems = Vec::new();
    for node in rust_files(&nodes) {
        let lines = read(&node.path).lines().count();
        if lines > MAX_LINES {
            problems.push(format!(
                "{}: {lines} lines, the limit is {MAX_LINES}",
                node.rel
            ));
        }
    }
    report(
        &problems,
        "split the file: move a group of items into a sibling module declared with `mod`",
    );
}

#[test]
fn source_files_open_with_a_purpose_header() {
    let nodes = tree();
    let mut problems = Vec::new();
    for node in rust_files(&nodes) {
        if header_of(&read(&node.path)).is_empty() {
            problems.push(format!(
                "{}: the first non-blank line is not a `//!` header",
                node.rel
            ));
        }
    }
    report(
        &problems,
        "open the file with `//!` lines whose first sentence says what the file is for",
    );
}

#[test]
fn source_files_cite_their_cases() {
    let nodes = tree();
    let mut problems = Vec::new();
    for node in rust_files(&nodes) {
        let text = read(&node.path);
        let header = header_of(&text);
        if !header.is_empty() && !header.iter().any(|line| line.starts_with("//! Cases:")) {
            problems.push(format!(
                "{}: its `//!` header has no `//! Cases:` line",
                node.rel
            ));
        }
    }
    report(
        &problems,
        "add `//! Cases: <ids>` to the header, or `//! Cases: none (<why>)` when no case pins the file",
    );
}

#[test]
fn every_folder_has_a_readme() {
    let nodes = tree();
    let mut problems = Vec::new();
    for node in nodes
        .iter()
        .filter(|node| node.is_dir && is_gated_folder(&node.rel))
    {
        if !has_readme(&node.path) {
            problems.push(format!("{}/: no README.md", node.rel));
        }
    }
    report(
        &problems,
        "add a README.md whose first paragraph says what the folder is",
    );
}

/// True when `dir` lists a file named exactly `README.md`. The names are compared as listed, so a `readme.md` does not
/// pass on a case-folding file system (design §4.4 Name matching).
fn has_readme(dir: &Path) -> bool {
    fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        .map(|entry| entry.expect("a folder entry is readable"))
        .any(|entry| entry.file_name().to_str() == Some("README.md") && entry.path().is_file())
}

#[test]
fn src_silences_no_lint() {
    let nodes = tree();
    let mut problems = Vec::new();
    for node in rust_files(&nodes) {
        if node.rel.starts_with("src/") {
            problems.extend(silencing_lines(&node.rel, &read(&node.path), None));
        }
    }
    report(
        &problems,
        "fix the code instead of silencing the lint: no #[allow] and no #[expect] under src/",
    );
}

#[test]
fn tests_allow_only_dead_code() {
    let nodes = tree();
    let mut problems = Vec::new();
    for node in rust_files(&nodes) {
        if node.rel.starts_with("tests/") {
            problems.extend(silencing_lines(
                &node.rel,
                &read(&node.path),
                Some(TEST_ALLOW),
            ));
        }
    }
    report(
        &problems,
        "tests/ may use #[allow(dead_code)] and nothing else: remove the other allow, or fix the code",
    );
}

/// Every file and folder under the roots, depth first, parents before children.
fn tree() -> Vec<Node> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut nodes = Vec::new();
    for root in ROOTS {
        visit(&manifest.join(root), manifest, &mut nodes);
    }
    nodes
}

/// Adds the entries under `dir` to `nodes`, depth first, with each entry before its own children.
fn visit(dir: &Path, manifest: &Path, nodes: &mut Vec<Node>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()));
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| entry.expect("a folder entry is readable").path())
        .collect();
    paths.sort();
    for path in paths {
        let is_dir = path.is_dir();
        let rel = path
            .strip_prefix(manifest)
            .expect("a walked path lies under the crate root")
            .to_string_lossy()
            .replace('\\', "/");
        nodes.push(Node {
            rel,
            path: path.clone(),
            is_dir,
        });
        if is_dir {
            visit(&path, manifest, nodes);
        }
    }
}

/// The Rust files the file gates judge: every .rs under src/ and tests/, minus fixture and self-test data.
fn rust_files(nodes: &[Node]) -> Vec<&Node> {
    nodes
        .iter()
        .filter(|node| {
            !node.is_dir && node.rel.ends_with(".rs") && !is_fixture_or_selftest(&node.rel)
        })
        .collect()
}

/// True when the README gate applies to a folder: not fixture data, not a self-test tree, not a case folder.
fn is_gated_folder(rel: &str) -> bool {
    !is_fixture_or_selftest(rel) && !is_case_folder_or_below(rel)
}

/// True for a fixture tree, whose files are sandbox data, and for the runner's deliberately broken trees.
fn is_fixture_or_selftest(rel: &str) -> bool {
    format!("/{rel}/").contains("/fixture/") || rel.starts_with("tests/runner-selftest/")
}

/// True for a case folder, tests/cases/<family>/<ID>, and everything under it: case data, not code.
fn is_case_folder_or_below(rel: &str) -> bool {
    rel.strip_prefix("tests/cases/")
        .is_some_and(|rest| rest.split('/').count() >= 2)
}

/// The `//!` lines at the top of a file, after any blank lines. Empty when the first non-blank line is not one.
fn header_of(text: &str) -> Vec<&str> {
    text.lines()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| line.starts_with("//!"))
        .collect()
}

/// True when a line opens a lint attribute that allows or expects a lint, such as `#[allow(unused)]`.
fn silences_a_lint(line: &str) -> bool {
    let line = line.trim_start();
    let attribute = line.starts_with("#[") || line.starts_with("#![");
    attribute && (line.contains("allow(") || line.contains("expect("))
}

/// One problem line per lint-silencing line of a file, except lines that start with the `excused` attribute.
fn silencing_lines(rel: &str, text: &str, excused: Option<&str>) -> Vec<String> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| silences_a_lint(line))
        .filter(|(_, line)| !excused.is_some_and(|attr| line.trim_start().starts_with(attr)))
        .map(|(index, line)| format!("{rel}:{}: {}", index + 1, line.trim()))
        .collect()
}

/// The text of a file. A file that cannot be read fails the gate, and is never skipped.
fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
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
