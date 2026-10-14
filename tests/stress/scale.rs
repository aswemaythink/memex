//! Scale tests: the verbs on large intent layers, timed against their budgets.
//! Cases: none (stress)
//!
//! The log store and the single large files (a 1 MB line, 50 MB files) are in `scale_logs`, which shares these helpers.

use std::fs;
use std::io::{self, Write as _};
use std::path::Path;
use std::process::Output;

use serde_json::Value;

use super::support::{
    cmd, load_avg, median_ms, min_ms, output_text, report, run_stdin, write_tree, Prng, Sandbox,
    TreeSpec,
};

/// The `.rs` files in each folder of a generated tree, next to its README.
const FILES_PER_FOLDER: usize = 9;

/// One mebibyte: the unit of the 50 MB inputs.
pub(super) const MIB: usize = 1024 * 1024;

/// The body of the entry that the random-bytes test logs.
const BODY: &str = "# Random entry\n\nThe log after the noise.\n";

/// Runs `memex args` in `sandbox` and returns its output. Panics, naming the arguments, unless it exits 0.
pub(super) fn run_ok(sandbox: &Sandbox, args: &[&str]) -> Output {
    let output = cmd(sandbox)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("memex did not start: {e}"));
    assert!(
        output.status.success(),
        "memex {args:?} exited with {:?}: {}",
        output.status.code(),
        output_text(&output.stderr)
    );
    output
}

/// Runs `memex args` and parses its stdout as one JSON document. Panics unless it exits 0 with JSON.
pub(super) fn json_of(sandbox: &Sandbox, args: &[&str]) -> Value {
    let output = run_ok(sandbox, args);
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("memex {args:?} printed no JSON: {e}"))
}

/// The collections of an `info --json` document: the built-in ones, the manifests and the log streams.
fn collections(info: &Value) -> impl Iterator<Item = &Value> {
    info["collections"].as_array().into_iter().flatten()
}

/// The READMEs, headers and cases of an `info --json` document, in that order.
pub(super) fn intent(info: &Value) -> [u64; 3] {
    ["readmes", "headers", "cases"].map(|name| {
        collections(info)
            .find(|c| c["name"] == name)
            .and_then(|c| c["count"].as_u64())
            .unwrap_or(0)
    })
}

/// The entries of all the log streams of an `info --json` document.
pub(super) fn log_total(info: &Value) -> u64 {
    collections(info)
        .filter(|c| c["kind"] == "log")
        .filter_map(|c| c["count"].as_u64())
        .sum()
}

/// The timings in `timed` (verb, ms, budget in ms) that are over their budget, each named with its numbers.
pub(super) fn over_budget(timed: &[(&str, f64, f64)]) -> Vec<String> {
    timed
        .iter()
        .filter(|(_, ms, budget)| ms > budget)
        .map(|(verb, ms, budget)| format!("{verb} took {ms:.1} ms, the budget is {budget:.0} ms"))
        .collect()
}

/// The median of three timed runs of `memex args` after one warm-up, in ms, printed with its label and the load average.
pub(super) fn time_verb(sandbox: &Sandbox, label: &str, args: &[&str]) -> f64 {
    let ms = median_ms(3, 1, || {
        run_ok(sandbox, args);
    });
    report(label, ms);
    ms
}

/// The fastest of five runs of `info` in `sandbox`, in ms, after one warm-up run.
pub(super) fn fastest_info(sandbox: &Sandbox) -> f64 {
    run_ok(sandbox, &["info"]);
    min_ms(5, || {
        run_ok(sandbox, &["info"]);
    })
}

/// Prints `text` and the load average to stderr directly, so the line shows without `--nocapture`.
pub(super) fn note(text: &str) {
    let _ = writeln!(io::stderr(), "{text} (load {})", load_avg());
}

/// Writes `bytes` to `path`, replacing any file there. Panics with the path when the write fails.
pub(super) fn put(path: &Path, bytes: impl AsRef<[u8]>) {
    fs::write(path, bytes).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}

/// Makes `dir` and its missing parents. Panics with the path when that fails.
pub(super) fn make_dir(dir: &Path) {
    fs::create_dir_all(dir).unwrap_or_else(|e| panic!("cannot make {}: {e}", dir.display()));
}

/// Writes `bytes` to `rel` under `sandbox`, making its folders first.
pub(super) fn write_in(sandbox: &Sandbox, rel: &str, bytes: impl AsRef<[u8]>) {
    let path = sandbox.path().join(rel);
    if let Some(dir) = path.parent() {
        make_dir(dir);
    }
    put(&path, bytes);
}

/// A sandbox holding a generated tree: `folders` folders of a README and nine `.rs` files, and `cases` cases in the root.
pub(super) fn tree_sandbox(seed: u64, folders: usize, cases: usize) -> Sandbox {
    let sandbox = Sandbox::new();
    let spec = TreeSpec {
        seed,
        folders,
        files_per_folder: FILES_PER_FOLDER,
        cases,
    };
    write_tree(sandbox.path(), &spec);
    sandbox
}

/// The READMEs, headers and cases that `tree_sandbox` writes for `folders` folders and `cases` cases.
fn generated(folders: usize, cases: usize) -> [u64; 3] {
    [
        folders as u64 + 1,
        (folders * FILES_PER_FOLDER) as u64,
        cases as u64,
    ]
}

/// 100k files (10k folders of a README and nine `.rs` files, 2k cases): `info` counts them all, and `info`, `search` and `show`
/// run within their budgets.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_100k_files() {
    let sandbox = tree_sandbox(1, 10_000, 2_000);
    let info = json_of(&sandbox, &["info", "--json"]);
    assert_eq!(
        intent(&info),
        generated(10_000, 2_000),
        "the counts of info"
    );
    let info_ms = time_verb(&sandbox, "info, 100k files", &["info"]);
    let search_ms = time_verb(&sandbox, "search cache, 100k files", &["search", "cache"]);
    let show_ms = time_verb(&sandbox, "show dir05000, 100k files", &["show", "dir05000"]);
    // show <dir> loads the same corpus as info, so it shares info's 1.5 s budget (coordinator ruling).
    let timed = [
        ("info", info_ms, 1_500.0),
        ("search", search_ms, 2_000.0),
        ("show", show_ms, 1_500.0),
    ];
    let breaches = over_budget(&timed);
    assert!(breaches.is_empty(), "over budget: {breaches:?}");
}

/// Linear scaling: `info` on 100k files takes at most 2.5 times as long as on 50k, by the ratio of the fastest of five runs.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_linear_50k_to_100k() {
    let half = tree_sandbox(2, 5_000, 1_000);
    let full = tree_sandbox(3, 10_000, 2_000);
    let small = fastest_info(&half);
    let large = fastest_info(&full);
    let ratio = large / small;
    report("info, 50k files, fastest of 5", small);
    report("info, 100k files, fastest of 5", large);
    note(&format!(
        "scale ratio 100k/50k = {ratio:.2} (target 2.2, budget 2.5)"
    ));
    assert!(
        ratio <= 2.5,
        "info grew {ratio:.2}x from 50k to 100k files, the budget is 2.5x"
    );
}

/// A folder of 20k files: `show <folder> --limit 50` prints `showing 50 of 20000` within 1 s.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_20k_files_in_one_folder() {
    let sandbox = Sandbox::new();
    write_in(
        &sandbox,
        "wide/README.md",
        "# Wide\n\nA folder of twenty thousand files.\n",
    );
    for index in 0..20_000 {
        let text = format!("//! Wide file {index}.\n\npub const V: u64 = {index};\n");
        write_in(&sandbox, &format!("wide/file{index:05}.rs"), text);
    }
    let args = ["show", "wide", "--limit", "50"];
    let shown = output_text(&run_ok(&sandbox, &args).stdout);
    assert!(
        shown.contains("showing 50 of 20000"),
        "the listing does not say so:\n{shown}"
    );
    let ms = time_verb(&sandbox, "show wide --limit 50, 20k files", &args);
    assert!(ms <= 1_000.0, "show took {ms:.1} ms, the budget is 1000 ms");
}

/// A chain of `depth` folders named `d`, each with a README that names its level, under a root README.
fn chain_sandbox(depth: usize) -> Sandbox {
    let sandbox = Sandbox::new();
    write_in(
        &sandbox,
        "README.md",
        "# Level 0\n\nThe root of the chain.\n",
    );
    let mut dir = sandbox.path().to_path_buf();
    for level in 1..=depth {
        dir.push("d");
        make_dir(&dir);
        put(
            &dir.join("README.md"),
            format!("# Level {level}\n\nLevel {level} of the chain.\n"),
        );
    }
    sandbox
}

/// The path of the deepest folder of a chain of `depth` folders.
fn deepest(depth: usize) -> String {
    vec!["d"; depth].join("/")
}

/// The number of entries in the `chain` of the first doc of a `show --json` document.
fn chain_len(shown: &Value) -> usize {
    shown["docs"][0]["chain"].as_array().map_or(0, Vec::len)
}

/// A 200-deep README chain: `show` of the deepest folder lists 200 chain entries, none with `--up 0`, within 0.3 s.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_deep_chain_200() {
    let sandbox = chain_sandbox(200);
    let folder = deepest(200);
    let shown = json_of(&sandbox, &["show", folder.as_str(), "--json"]);
    assert_eq!(chain_len(&shown), 200, "the chain of the deepest folder");
    let flat = json_of(&sandbox, &["show", folder.as_str(), "--up", "0", "--json"]);
    assert_eq!(chain_len(&flat), 0, "--up 0 prints no chain");
    let ms = time_verb(
        &sandbox,
        "show deepest of 200 READMEs",
        &["show", folder.as_str()],
    );
    assert!(ms <= 300.0, "show took {ms:.1} ms, the budget is 300 ms");
}

/// A 400-deep README chain: `show` of the deepest folder and `info` exit 0, with no panic.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_deep_chain_400() {
    let sandbox = chain_sandbox(400);
    let folder = deepest(400);
    run_ok(&sandbox, &["show", folder.as_str()]);
    run_ok(&sandbox, &["info"]);
}

/// 1000 random-byte files, half `.rs` and half `.md` (fixed seed): every verb exits 0.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_random_bytes_every_verb_exits_0() {
    let sandbox = Sandbox::new();
    let init = sandbox.memex_init();
    assert!(
        init.status.success(),
        "memex init failed: {}",
        output_text(&init.stderr)
    );
    let mut rng = Prng::new(7);
    let mut files = Vec::new();
    for index in 0..1_000 {
        let ext = if index % 2 == 0 { "rs" } else { "md" };
        let len = rng.below(8_192);
        let bytes: Vec<u8> = (0..len).map(|_| rng.next_u64() as u8).collect();
        let rel = format!("noise/file{index:04}.{ext}");
        write_in(&sandbox, &rel, bytes);
        files.push(rel);
    }
    run_ok(&sandbox, &["info"]);
    run_ok(&sandbox, &["info", "--json"]);
    run_ok(&sandbox, &["search", "cache"]);
    run_ok(&sandbox, &["search", "--in", "headers"]);
    run_ok(&sandbox, &["show", "."]);
    run_ok(&sandbox, &["show", "noise"]);
    let mut shown = vec!["show"];
    shown.extend(files.iter().map(String::as_str));
    run_ok(&sandbox, &shown);
    run_ok(&sandbox, &["init"]);
    let logged = run_stdin(cmd(&sandbox).args(["log", "stream-00"]), BODY);
    assert!(
        logged.status.success(),
        "memex log failed: {}",
        output_text(&logged.stderr)
    );
}
