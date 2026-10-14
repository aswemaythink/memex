//! Concurrency tests: many memex processes at once on one store, run by the stress suite.
//! Each test starts real processes of the built binary behind a barrier, so that they race for the same files.
//! Cases: none (stress)

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Barrier};
use std::thread;

use crate::support::{cmd, output_text, run_stdin, Sandbox};

/// The streams that the prune tests write into and prune.
const STREAMS: [&str; 3] = ["impl", "review", "drift"];
/// The stem of the same-second entries: MEMEX_NOW's stamp, then the slug of their shared title.
const SAME_STEM: &str = "20261014-120000-same-second";
/// The anchor is current, so no prune takes it, and the readers show it.
const ANCHOR: &str = "impl:20261014-120000-anchor";
/// Three months before MEMEX_NOW, so the default keep_logs of 3d prunes these entries.
const EXPIRED_STAMP: &str = "20260701-120000";

/// One memex process for `run_together`: its command, and the text it reads on stdin (None for a reader).
struct Job(Command, Option<String>);

/// The jobs of the worktree test, with each writer's name and body in the same order.
type Plan = (Vec<Job>, Vec<String>, Vec<String>);

/// Ten rounds of 50 processes and one of 200 log one title in one second: each ref is distinct, each body intact.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn conc_same_second() {
    for _ in 0..10 {
        same_second_round(50);
    }
    same_second_round(200);
}

/// `writers` processes log one title at MEMEX_NOW. Each takes the next free name: the stem, then -2 and up.
fn same_second_round(writers: usize) {
    let sandbox = repo();
    let bodies: Vec<String> = (1..=writers)
        .map(|n| format!("# Same second\n\nWriter {n:03} wrote this line.\n"))
        .collect();
    let jobs: Vec<Job> = bodies
        .iter()
        .map(|body| log_job(&sandbox, "impl", body, None))
        .collect();
    let written = check_writes(sandbox.path(), &run_together(jobs), &bodies);
    let mut want = BTreeSet::from([format!("impl/{SAME_STEM}.md")]);
    want.extend((2..=writers).map(|n| format!("impl/{SAME_STEM}-{n}.md")));
    assert_eq!(written, want, "the refs differ");
    assert_store(sandbox.path(), &want);
}

/// Writers and readers run at once over 5,000 expired entries, then 20 rounds where writers land in pruned folders.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn conc_log_while_prune() {
    prune_under_load();
    for _ in 0..20 {
        emptied_folder_round();
    }
}

/// 20 writers and 10 readers start together over 5,000 expired entries. The store must then hold the anchor and the 20.
fn prune_under_load() {
    let sandbox = repo();
    let root = sandbox.path();
    let anchor = log_job(&sandbox, "impl", "# Anchor\n\nThe anchor stays.\n", None);
    assert_clean(&start(anchor, &Barrier::new(1)), "the anchor's memex log");
    write_expired(root, 5000);
    let bodies: Vec<String> = (0..20)
        .map(|n| format!("# New entry {n:02}\n\nWriter {n:02} logged during the prune.\n"))
        .collect();
    let mut jobs = writers(&sandbox, &bodies);
    jobs.extend((0..10).map(|n| reader_job(&sandbox, n)));
    let outputs = run_together(jobs);
    let (writes, reads) = outputs.split_at(bodies.len());
    for (n, output) in reads.iter().enumerate() {
        assert_clean(output, &format!("reader {n}"));
    }
    let mut want = check_writes(root, writes, &bodies);
    want.insert(store_key(ANCHOR));
    assert_store(root, &want);
}

/// 90 expired entries, then 40 writers log into the same streams while the prunes they start empty them.
fn emptied_folder_round() {
    let sandbox = repo();
    write_expired(sandbox.path(), 90);
    let bodies: Vec<String> = (0..40)
        .map(|n| format!("# Round {n:02}\n\nWriter {n:02} in a folder that is being pruned.\n"))
        .collect();
    let outputs = run_together(writers(&sandbox, &bodies));
    let want = check_writes(sandbox.path(), &outputs, &bodies);
    assert_store(sandbox.path(), &want);
}

/// Ten writers in the main checkout and in each of three linked worktrees log at once into one store. Each entry names
/// its writer's worktree, and all 40 remain after the worktrees are removed.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn conc_worktrees() {
    let sandbox = repo();
    let root = sandbox.path();
    let holder = tempfile::Builder::new()
        .prefix("memex-stress-wt-")
        .tempdir()
        .unwrap_or_else(|e| panic!("cannot make a folder for the worktrees: {e}"));
    let worktrees = ["wt-1", "wt-2", "wt-3"].map(|name| holder.path().join(name));
    for path in &worktrees {
        let arg = path.to_string_lossy().into_owned();
        let added = git(root, &["worktree", "add", "--detach", &arg]);
        must_succeed(&added, "git worktree add");
    }
    let (jobs, names, bodies) = worktree_jobs(&sandbox, &worktrees);
    let outputs = run_together(jobs);
    let want = check_writes(root, &outputs, &bodies);
    let refs: Vec<String> = outputs.iter().map(printed_ref).collect();
    for (reference, name) in refs.iter().zip(&names) {
        assert_worktree(root, reference, name);
    }
    assert_store(root, &want);
    for path in &worktrees {
        let arg = path.to_string_lossy().into_owned();
        let removed = git(root, &["worktree", "remove", &arg]);
        must_succeed(&removed, "git worktree remove");
    }
    assert_store(root, &want);
    for (reference, body) in refs.iter().zip(&bodies) {
        assert_body(root, reference, body);
    }
}

/// The 40 writers of the worktree test: ten from the main checkout, named `main`, then ten from each linked worktree.
fn worktree_jobs(sandbox: &Sandbox, worktrees: &[PathBuf]) -> Plan {
    let mut sources: Vec<(String, Option<&Path>)> = vec![("main".to_string(), None)];
    for (k, path) in worktrees.iter().enumerate() {
        sources.push((format!("wt-{}", k + 1), Some(path.as_path())));
    }
    let (mut jobs, mut names, mut bodies) = (Vec::new(), Vec::new(), Vec::new());
    for (name, dir) in &sources {
        for n in 1..=10 {
            let body = format!("# Entry from {name} {n:02}\n\nWriter {n:02} ran in {name}.\n");
            jobs.push(log_job(sandbox, "impl", &body, *dir));
            names.push(name.clone());
            bodies.push(body);
        }
    }
    (jobs, names, bodies)
}

/// A fresh git repository with one empty commit and `memex init` run in it, as a real project is.
fn repo() -> Sandbox {
    let sandbox = Sandbox::new();
    must_succeed(&sandbox.git_init(), "git init");
    let commit = git(sandbox.path(), &["commit", "--allow-empty", "-qm", "init"]);
    must_succeed(&commit, "git commit");
    must_succeed(&sandbox.memex_init(), "memex init");
    sandbox
}

/// Runs `git args` in `dir` with a fixed identity, and without GIT_* variables or the system and global config.
fn git(dir: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("git");
    for (name, _) in std::env::vars_os() {
        let upper = name.to_string_lossy().to_ascii_uppercase();
        if upper.starts_with("GIT_") {
            command.env_remove(name);
        }
    }
    command
        .args(["-c", "user.name=stress", "-c", "user.email=stress@x.test"])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", dir.join(".gitconfig"));
    command
        .output()
        .unwrap_or_else(|e| panic!("git did not start: {e}"))
}

/// Starts every job's process at once: a thread per job waits at a barrier, then runs its command to the end.
fn run_together(jobs: Vec<Job>) -> Vec<Output> {
    let barrier = Arc::new(Barrier::new(jobs.len()));
    let threads: Vec<_> = jobs
        .into_iter()
        .map(|job| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || start(job, &barrier))
        })
        .collect();
    threads
        .into_iter()
        .map(|thread| {
            thread
                .join()
                .unwrap_or_else(|_| panic!("a process thread panicked"))
        })
        .collect()
}

/// Waits at `barrier`, then runs the job's process to the end and returns its output.
fn start(job: Job, barrier: &Barrier) -> Output {
    let Job(mut command, stdin) = job;
    barrier.wait();
    match stdin {
        Some(text) => run_stdin(&mut command, &text),
        None => command
            .output()
            .unwrap_or_else(|e| panic!("did not start: {e}")),
    }
}

/// A writer: `memex log <stream>` with `body` on stdin, run from `dir` when one is given.
fn log_job(sandbox: &Sandbox, stream: &str, body: &str, dir: Option<&Path>) -> Job {
    let mut command = cmd(sandbox);
    command.arg("log").arg(stream);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    Job(command, Some(body.to_string()))
}

/// One writer per body, round-robin over the streams.
fn writers(sandbox: &Sandbox, bodies: &[String]) -> Vec<Job> {
    let mut jobs = Vec::new();
    for (n, body) in bodies.iter().enumerate() {
        jobs.push(log_job(sandbox, STREAMS[n % STREAMS.len()], body, None));
    }
    jobs
}

/// A reader, in turn: a search, an info, or a show of the anchor. It reads no stdin.
fn reader_job(sandbox: &Sandbox, n: usize) -> Job {
    let mut command = cmd(sandbox);
    let args: &[&str] = match n % 3 {
        0 => &["search", "anchor"],
        1 => &["info"],
        _ => &["show", ANCHOR],
    };
    command.args(args);
    Job(command, None)
}

/// Writes `count` expired entries spread over the streams. The next write prunes each one.
fn write_expired(root: &Path, count: usize) {
    for n in 0..count {
        let stream = STREAMS[n % STREAMS.len()];
        let dir = store(root).join(stream);
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let text = format!("---\ntime: 2026-07-01T12:00:00Z\n---\n# Expired {n:05}\n");
        let path = dir.join(format!("{EXPIRED_STAMP}-expired-{n:05}.md"));
        fs::write(&path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
}

/// Checks the writers: each exited 0 with an empty stderr, and its entry holds its body. Returns the keys they wrote.
fn check_writes(root: &Path, outputs: &[Output], bodies: &[String]) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for (index, (output, body)) in outputs.iter().zip(bodies).enumerate() {
        assert_clean(output, &format!("writer {index}"));
        let reference = printed_ref(output);
        assert_body(root, &reference, body);
        keys.insert(store_key(&reference));
    }
    assert_eq!(keys.len(), bodies.len(), "two writers printed one ref");
    keys
}

/// Panics unless the store holds exactly the files in `want`. A hidden name such as a temp file is extra.
fn assert_store(root: &Path, want: &BTreeSet<String>) {
    let found = store_files(root);
    let diff: Vec<&String> = found.symmetric_difference(want).take(5).collect();
    assert!(diff.is_empty(), "the store differs at {diff:?}");
}

/// Every file in the store as `stream/name`, hidden names included.
fn store_files(root: &Path) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    for stream in list(&store(root)) {
        for name in list(&store(root).join(&stream)) {
            files.insert(format!("{stream}/{name}"));
        }
    }
    files
}

/// The names in `dir`, sorted. Panics when the folder cannot be listed.
fn list(dir: &Path) -> Vec<String> {
    let listing = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    let mut names: Vec<String> = listing
        .map(|entry| entry.unwrap_or_else(|e| panic!("{}: {e}", dir.display())))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Panics unless the entry `reference` ends with `body`: a body is stored whole, after its frontmatter.
fn assert_body(root: &Path, reference: &str, body: &str) {
    let path = entry_file(root, reference);
    let text = read(&path);
    let shown = path.display();
    assert!(text.ends_with(body), "{shown} lacks its body\n{text}");
}

/// Panics unless the entry `reference` names its writer's worktree: `wt-1` and so on, or none for `main`.
fn assert_worktree(root: &Path, reference: &str, name: &str) {
    let path = entry_file(root, reference);
    let text = read(&path);
    let field = text.lines().find_map(|l| l.strip_prefix("worktree: "));
    let ok = field.map_or(name == "main", |v| v.rsplit('/').next() == Some(name));
    let shown = path.display();
    assert!(ok, "{shown} names {field:?} for {name}");
}

/// Panics unless `output` exited 0 and wrote nothing to stderr. `what` names the process in the message.
fn assert_clean(output: &Output, what: &str) {
    let stderr = output_text(&output.stderr);
    let ok = output.status.success() && stderr.is_empty();
    assert!(ok, "{what} exited {}: {stderr:?}", output.status);
}

/// Panics unless `output` exited 0. `what` names the command, and its stderr says why not.
fn must_succeed(output: &Output, what: &str) {
    let stderr = output_text(&output.stderr);
    assert!(output.status.success(), "{what} failed: {stderr}");
}

/// The ref that a `log` process printed on stdout, without its newline.
fn printed_ref(output: &Output) -> String {
    output_text(&output.stdout).trim_end().to_string()
}

fn store(root: &Path) -> PathBuf {
    root.join(".memex").join("logs")
}

/// The store key of the entry `reference`: `stream/id.md`, from the ref `stream:id`.
fn store_key(reference: &str) -> String {
    format!("{}.md", reference.replacen(':', "/", 1))
}

fn entry_file(root: &Path, reference: &str) -> PathBuf {
    store(root).join(store_key(reference))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}
