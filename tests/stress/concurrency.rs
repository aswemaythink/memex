//! Concurrency tests: many memex processes at once on one store, run by the stress suite.
//! Each test starts real processes of the built binary behind a barrier, so that they race for the same files.
//! The worktree test is in conc_worktrees.rs, which uses the helpers this file shares with it.
//! Cases: none (stress)

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Barrier, Mutex, PoisonError};
use std::thread;

use crate::support::{cmd, output_text, Sandbox};

/// The streams that the prune tests write into and prune.
const STREAMS: [&str; 3] = ["impl", "review", "drift"];
/// The stem of the same-second entries: MEMEX_NOW's stamp, then the slug of their shared title.
const SAME_STEM: &str = "20261014-120000-same-second";
/// The anchor is current, so no prune takes it, and the readers show it.
const ANCHOR: &str = "impl:20261014-120000-anchor";
const ANCHOR_BODY: &str = "# Anchor\n\nThe anchor stays.\n";
/// Three months before MEMEX_NOW, so the keep_logs of 3d that `write_expired` sets prunes these entries.
const EXPIRED_STAMP: &str = "20260701-120000";
/// How many children may hold their stdio files open in this process at once: 3 descriptors each, well under 256.
const SLOTS: usize = 32;

/// One memex process for `run_together`: its command, and the text it reads on stdin (None for a reader).
pub(crate) struct Job(Command, Option<String>);

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
    let anchor = run_together(vec![log_job(&sandbox, "impl", ANCHOR_BODY, None)]);
    assert_clean(&anchor[0], "the anchor's memex log");
    write_expired(root, 5000);
    let bodies: Vec<String> = (0..20)
        .map(|n| format!("# New entry {n:02}\n\nWriter {n:02} logged during the prune.\n"))
        .collect();
    let mut jobs = writers(&sandbox, &bodies);
    jobs.extend((0..10).map(|n| reader_job(&sandbox, n)));
    let outputs = run_together(jobs);
    let (writes, reads) = outputs.split_at(bodies.len());
    for output in reads {
        assert_clean(output, "reader");
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

/// A fresh git repository with one empty commit and `memex init` run in it, as a real project is.
pub(crate) fn repo() -> Sandbox {
    let sandbox = Sandbox::new();
    must_succeed(&sandbox.git_init(), "git init");
    let commit = git(sandbox.path(), &["commit", "--allow-empty", "-qm", "init"]);
    must_succeed(&commit, "git commit");
    must_succeed(&sandbox.memex_init(), "memex init");
    sandbox
}

/// Runs `git args` in `dir` with a fixed identity, and without the GIT_DIR, GIT_WORK_TREE and GIT_INDEX_FILE a hook sets.
pub(crate) fn git(dir: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("git");
    command
        .args(["-c", "user.name=stress", "-c", "user.email=stress@x.test"])
        .args(args)
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", dir.join(".gitconfig"));
    command
        .output()
        .unwrap_or_else(|e| panic!("git did not start: {e}"))
}

/// Starts every job's process at once: a thread per job waits at a barrier, then runs its command to the end.
pub(crate) fn run_together(jobs: Vec<Job>) -> Vec<Output> {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("cannot make a folder: {e}"));
    let barrier = Arc::new(Barrier::new(jobs.len()));
    let slots = Arc::new((0..SLOTS).map(|_| Mutex::new(())).collect::<Vec<_>>());
    let mut threads = Vec::new();
    for (n, job) in jobs.into_iter().enumerate() {
        let barrier = Arc::clone(&barrier);
        let slots = Arc::clone(&slots);
        let dir = dir.path().to_path_buf();
        threads.push(thread::spawn(move || {
            start(job, &barrier, &slots[n % SLOTS], &dir, n)
        }));
    }
    let mut outputs = Vec::new();
    for thread in threads {
        outputs.push(
            thread
                .join()
                .unwrap_or_else(|_| panic!("a thread panicked")),
        );
    }
    outputs
}

/// Runs the job's process to the end. Its stdin and outputs are files, opened under the thread's slot only while the
/// child is spawned, so at most `SLOTS` children hold descriptors in this process at once.
fn start(job: Job, barrier: &Barrier, slot: &Mutex<()>, dir: &Path, n: usize) -> Output {
    let Job(command, stdin) = job;
    let input = dir.join(format!("{n}.in"));
    let (out, err) = (dir.join(format!("{n}.out")), dir.join(format!("{n}.err")));
    fs::write(&input, stdin.unwrap_or_default()).unwrap_or_else(|e| panic!("stdin: {e}"));
    barrier.wait();
    let mut child = {
        let _slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
        let mut command = command;
        command.stdin(stdio(&input, false));
        command.stdout(stdio(&out, true));
        command.stderr(stdio(&err, true));
        command
            .spawn()
            .unwrap_or_else(|e| panic!("did not start: {e}"))
    };
    let status = child
        .wait()
        .unwrap_or_else(|e| panic!("did not finish: {e}"));
    let stdout = read(&out).into_bytes();
    let stderr = read(&err).into_bytes();
    Output {
        status,
        stdout,
        stderr,
    }
}

/// A file as a child's stdio: opened for reading when `write` is false, else created.
fn stdio(path: &Path, write: bool) -> Stdio {
    let file = if write {
        File::create(path)
    } else {
        File::open(path)
    };
    Stdio::from(file.unwrap_or_else(|e| panic!("{}: {e}", path.display())))
}

/// A writer: `memex log <stream>` with `body` on stdin, run from `dir` when one is given.
pub(crate) fn log_job(sandbox: &Sandbox, stream: &str, body: &str, dir: Option<&Path>) -> Job {
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

/// Writes `count` expired entries spread over the streams, and sets keep_logs to 3d, since the default has no age limit. The next
/// write prunes each one.
fn write_expired(root: &Path, count: usize) {
    let config = root.join(".memex").join("config.json");
    fs::write(&config, "{\"keep_logs\": \"3d\"}\n")
        .unwrap_or_else(|e| panic!("{}: {e}", config.display()));
    for n in 0..count {
        let dir = store(root).join(STREAMS[n % STREAMS.len()]);
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let text = format!("---\ntime: 2026-07-01T12:00:00Z\n---\n# Expired {n:05}\n");
        let path = dir.join(format!("{EXPIRED_STAMP}-expired-{n:05}.md"));
        fs::write(&path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
}

/// Checks the writers: each exited 0 with an empty stderr, and its entry holds its body. Returns the keys they wrote.
pub(crate) fn check_writes(root: &Path, outputs: &[Output], bodies: &[String]) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for (index, (output, body)) in outputs.iter().zip(bodies).enumerate() {
        assert_clean(output, &format!("writer {index}"));
        let reference = printed_ref(output);
        assert_body(root, &reference, body);
        keys.insert(store_key(&reference));
    }
    keys
}

/// Panics unless the store holds exactly the files in `want`. A hidden name such as a temp file is extra.
pub(crate) fn assert_store(root: &Path, want: &BTreeSet<String>) {
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
pub(crate) fn assert_body(root: &Path, reference: &str, body: &str) {
    let path = entry_file(root, reference);
    let text = read(&path);
    let shown = path.display();
    assert!(text.ends_with(body), "{shown} lacks its body\n{text}");
}

/// Panics unless `output` exited 0 and wrote nothing to stderr. `what` names the process in the message.
pub(crate) fn assert_clean(output: &Output, what: &str) {
    let stderr = output_text(&output.stderr);
    let ok = output.status.success() && stderr.is_empty();
    assert!(ok, "{what} exited {}: {stderr:?}", output.status);
}

/// Panics unless `output` exited 0. `what` names the command, and its stderr says why not.
pub(crate) fn must_succeed(output: &Output, what: &str) {
    let stderr = output_text(&output.stderr);
    assert!(output.status.success(), "{what} failed: {stderr}");
}

/// The ref that a `log` process printed on stdout, without its newline.
pub(crate) fn printed_ref(output: &Output) -> String {
    output_text(&output.stdout).trim_end().to_string()
}

fn store(root: &Path) -> PathBuf {
    root.join(".memex").join("logs")
}

/// The store key of the entry `reference`: `stream/id.md`, from the ref `stream:id`.
fn store_key(reference: &str) -> String {
    format!("{}.md", reference.replacen(':', "/", 1))
}

pub(crate) fn entry_file(root: &Path, reference: &str) -> PathBuf {
    store(root).join(store_key(reference))
}

pub(crate) fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}
