//! Worktree concurrency test: writers in linked worktrees share one store with the main checkout.
//! Ten writers in the main checkout and in each of three linked worktrees log at once. Each entry must name its writer's
//! worktree in its `worktree:` field, and `info` run from inside any worktree must count all 40 entries.
//! Cases: none (stress)

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::concurrency::{
    assert_body, assert_clean, assert_store, check_writes, entry_file, git, log_job, must_succeed,
    printed_ref, read, repo, run_together, Job,
};
use crate::support::{cmd, Sandbox};

/// The writers of the worktree test, in the same order: each job, the body it logs and the `worktree:` field its entry carries.
struct Plan {
    jobs: Vec<Job>,
    bodies: Vec<String>,
    /// The worktree's path as memex prints it, or None for a writer in the main checkout, whose entry has no such field.
    homes: Vec<Option<String>>,
}

/// Ten writers in the main checkout and in each linked worktree log at once. Each entry names its own worktree, `info` run
/// from inside any worktree counts all 40, and all 40 survive `git worktree remove`.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn conc_worktrees() {
    let sandbox = repo();
    let root = sandbox.path();
    let holder = tempfile::tempdir().unwrap_or_else(|e| panic!("cannot make a folder: {e}"));
    let worktrees = ["wt-1", "wt-2", "wt-3"].map(|name| holder.path().join(name));
    for path in &worktrees {
        let arg = path.to_string_lossy().into_owned();
        must_succeed(&git(root, &["worktree", "add", "--detach", &arg]), "add");
    }
    let plan = worktree_jobs(&sandbox, &worktrees);
    let outputs = run_together(plan.jobs);
    let want = check_writes(root, &outputs, &plan.bodies);
    let refs: Vec<String> = outputs.iter().map(printed_ref).collect();
    assert_store(root, &want);
    assert_homes(root, &refs, &plan.homes);
    for path in &worktrees {
        assert_info_count(&sandbox, path, "impl", 40);
    }
    for path in &worktrees {
        let arg = path.to_string_lossy().into_owned();
        must_succeed(&git(root, &["worktree", "remove", &arg]), "remove");
    }
    assert_store(root, &want);
    for (reference, body) in refs.iter().zip(&plan.bodies) {
        assert_body(root, reference, body);
    }
}

/// The writers of the worktree test: ten from the main checkout, then ten from each linked worktree.
fn worktree_jobs(sandbox: &Sandbox, worktrees: &[PathBuf]) -> Plan {
    let mut sources: Vec<(String, Option<&Path>)> = vec![("main".to_string(), None)];
    for (k, path) in worktrees.iter().enumerate() {
        sources.push((format!("wt-{}", k + 1), Some(path.as_path())));
    }
    let mut plan = Plan {
        jobs: Vec::new(),
        bodies: Vec::new(),
        homes: Vec::new(),
    };
    for (name, dir) in sources {
        for n in 1..=10 {
            let body = format!("# Entry from {name} {n:02}\n\nWriter {n:02} ran in {name}.\n");
            plan.jobs.push(log_job(sandbox, "impl", &body, dir));
            plan.bodies.push(body);
            plan.homes.push(dir.map(printed_path));
        }
    }
    plan
}

/// The path that memex prints in a `worktree:` field: the worktree's canonical path, with `/` separators on Windows.
fn printed_path(path: &Path) -> String {
    let canonical = dunce::canonicalize(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let text = canonical.to_string_lossy().into_owned();
    if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text
    }
}

/// Panics unless each entry's `worktree:` field is the path of the worktree that wrote it. An entry written from the main
/// checkout has no such field.
fn assert_homes(root: &Path, refs: &[String], homes: &[Option<String>]) {
    for (reference, home) in refs.iter().zip(homes) {
        let path = entry_file(root, reference);
        let field = frontmatter_field(&read(&path), "worktree");
        assert_eq!(
            field,
            *home,
            "{} has the wrong worktree field",
            path.display()
        );
    }
}

/// The value of `key` in the frontmatter of `text`: the lines between its first two `---` lines. None when the key is absent.
fn frontmatter_field(text: &str, key: &str) -> Option<String> {
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("---"), "the entry has no frontmatter");
    let prefix = format!("{key}: ");
    lines
        .take_while(|line| *line != "---")
        .find_map(|line| line.strip_prefix(prefix.as_str()).map(str::to_string))
}

/// Panics unless `memex info --json`, run in `dir`, exits 0 and counts `want` entries in the log stream `stream`.
fn assert_info_count(sandbox: &Sandbox, dir: &Path, stream: &str, want: u64) {
    let mut command = cmd(sandbox);
    command.args(["info", "--json"]).current_dir(dir);
    let output = command
        .output()
        .unwrap_or_else(|e| panic!("memex info did not start: {e}"));
    assert_clean(&output, &format!("info in {}", dir.display()));
    let document: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("info --json is not JSON: {e}"));
    let count = document["collections"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|item| item["kind"] == "log" && item["name"] == stream)
        .and_then(|item| item["count"].as_u64());
    assert_eq!(
        count,
        Some(want),
        "info in {} counts the stream {stream} wrongly",
        dir.display()
    );
}
