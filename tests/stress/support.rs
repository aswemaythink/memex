//! Shared support for the stress suite: the binary, sandboxes, seeded trees and timings.
//!
//! `bin()` is the memex binary this run was built with. `Sandbox::new()` is an empty folder with a unique name under the
//! system temp dir, removed when the value drops; its `git_init()` and `memex_init()` run `git init -q` and `memex init` in
//! it. `cmd(&sandbox)` is a `memex` command whose cwd and HOME are the sandbox, whose inherited MEMEX_*, GIT_* and agent
//! variables are removed, and whose MEMEX_NOW is `NOW`. `run_stdin(&mut command, text)` runs a command with `text` on its
//! stdin, and `output_text(bytes)` decodes its output. `write_tree(root, &TreeSpec)` writes a seeded intent layer: folders
//! with a README.md, `.rs` files with `//!` headers that cite case ids, and a CASES.md. `median_ms(n, warm, f)` and
//! `min_ms(n, f)` time a closure, and `report(label, ms)` prints a timing with `load_avg()` to stderr, so a noisy machine
//! shows in the output.
//!
//! Helpers panic with a message when the environment is broken (a spawn or a write fails); none returns an error for a test
//! to ignore. The only randomness is `Prng`, seeded from `TreeSpec::seed`, so one spec always writes one tree.
//! Cases: none (stress)

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Instant;

/// The clock every stress run sees: MEMEX_NOW, fixed so that refs and stamps match on every run.
pub const NOW: &str = "2026-10-14T12:00:00Z";

/// The cases in one `## Group` heading of a generated CASES.md.
const GROUP: usize = 100;

/// The words that generated text is made of.
const VOCAB: [&str; 24] = [
    "atom", "cache", "cite", "claim", "dense", "folder", "header", "index", "marker", "merge",
    "parse", "plan", "quiet", "ref", "render", "root", "stable", "stamp", "token", "trace", "walk",
    "window", "write", "yield",
];

/// The memex binary that this test run was built with.
pub fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_memex"))
}

/// A scratch folder for one stress run: a unique name under the system temp dir, removed when the value drops.
pub struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    /// Makes an empty sandbox. Panics if the temp dir cannot be written.
    pub fn new() -> Sandbox {
        let dir = tempfile::Builder::new()
            .prefix("memex-stress-")
            .tempdir()
            .unwrap_or_else(|e| panic!("cannot make a sandbox: {e}"));
        Sandbox { dir }
    }

    /// The sandbox folder: the cwd of every command from `cmd`, and its HOME.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Runs `git init -q` in the sandbox, with the scrubbed environment.
    pub fn git_init(&self) -> Output {
        let mut git = Command::new("git");
        git.args(["init", "-q"]).current_dir(self.path());
        scrub(&mut git, self.path());
        run_to_end(&mut git, "git init")
    }

    /// Runs `memex init` in the sandbox. Call `git_init` first to make the sandbox a repository, as a real project is.
    pub fn memex_init(&self) -> Output {
        run_to_end(cmd(self).arg("init"), "memex init")
    }
}

/// A `memex` command for the sandbox: its cwd and HOME are the sandbox, its MEMEX_* variables are removed except MEMEX_NOW
/// (fixed at `NOW`), and PATH is inherited. Add arguments, then run it with `output()` or `run_stdin`.
pub fn cmd(sandbox: &Sandbox) -> Command {
    let mut command = Command::new(bin());
    command.current_dir(sandbox.path());
    scrub(&mut command, sandbox.path());
    command
}

/// Scrubs `command`'s environment: the inherited MEMEX_*, GIT_*, CLAUDE* and AI_AGENT variables are removed, and HOME,
/// the git config files and the C locale are set. PATH is kept. Names compare upper-cased, so Windows spellings match too.
fn scrub(command: &mut Command, home: &Path) {
    for (name, _) in std::env::vars_os() {
        let upper = name.to_string_lossy().to_ascii_uppercase();
        let scrubbed = upper.starts_with("MEMEX_")
            || upper.starts_with("GIT_")
            || upper.starts_with("CLAUDE")
            || upper == "AI_AGENT";
        if scrubbed {
            command.env_remove(&name);
        }
    }
    command
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", home.join(".gitconfig"))
        .env("MEMEX_NOW", NOW)
        .env("LC_ALL", "C")
        .env("LANG", "C");
}

/// Runs `command` with `input` on its stdin and returns its output. The input is written from a thread of its own, so a
/// large input cannot deadlock against a child that is blocked on a full stdout pipe. Panics if the command cannot start.
pub fn run_stdin(command: &mut Command, input: &str) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("the command did not start: {e}"));
    let mut stdin = child
        .stdin
        .take()
        .unwrap_or_else(|| panic!("the command has no piped stdin"));
    let bytes = input.as_bytes().to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&bytes));
    let output = child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("the command did not finish: {e}"));
    // A child that exits without reading its input closes the pipe, which fails the write. That is not an error here.
    let _ = writer.join();
    output
}

/// Runs `command` to the end and returns its output. Panics with `what` when the command cannot start.
fn run_to_end(command: &mut Command, what: &str) -> Output {
    command
        .output()
        .unwrap_or_else(|e| panic!("{what} did not start: {e}"))
}

/// Decodes a child's output bytes as text, replacing invalid UTF-8.
pub fn output_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The median of `n` timed runs of `f` in milliseconds, after `warm` untimed runs. For an even `n` it is the upper middle.
pub fn median_ms(n: usize, warm: usize, mut f: impl FnMut()) -> f64 {
    for _ in 0..warm {
        f();
    }
    let mut times = timed(n, &mut f);
    times.sort_by(f64::total_cmp);
    times[times.len() / 2]
}

/// The fastest of `n` timed runs of `f` in milliseconds.
pub fn min_ms(n: usize, mut f: impl FnMut()) -> f64 {
    timed(n, &mut f).into_iter().fold(f64::INFINITY, f64::min)
}

/// The time of each of `n` runs of `f` in milliseconds, in run order.
fn timed(n: usize, f: &mut impl FnMut()) -> Vec<f64> {
    assert!(n > 0, "a timing needs at least one run");
    (0..n)
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect()
}

/// The 1, 5 and 15 minute load averages, or `unknown` where this OS has no source for them.
pub fn load_avg() -> String {
    load_os().unwrap_or_else(|| "unknown".to_string())
}

/// Prints one timing with the load average to stderr. It writes to the stream directly, so the line shows even without
/// `--nocapture`, and a noisy run says so beside its numbers.
pub fn report(label: &str, ms: f64) {
    let _ = writeln!(io::stderr(), "{label}: {ms:.1} ms (load {})", load_avg());
}

/// The load averages that `sysctl -n vm.loadavg` prints on macOS, which wraps them in braces.
#[cfg(target_os = "macos")]
fn load_os() -> Option<String> {
    let output = Command::new("sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let fields: Vec<&str> = text
        .split_whitespace()
        .filter(|field| !matches!(*field, "{" | "}"))
        .collect();
    (!fields.is_empty()).then(|| fields.join(" "))
}

/// The first three fields of `/proc/loadavg`, which are the 1, 5 and 15 minute load averages.
#[cfg(target_os = "linux")]
fn load_os() -> Option<String> {
    let text = fs::read_to_string("/proc/loadavg").ok()?;
    let fields: Vec<&str> = text.split_whitespace().take(3).collect();
    (!fields.is_empty()).then(|| fields.join(" "))
}

/// Where this OS has no load source, the timing lines say `unknown`.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn load_os() -> Option<String> {
    None
}

/// The shape of a generated intent layer. The seed decides every generated byte, so one spec always writes one tree.
#[derive(Debug, Clone, Copy)]
pub struct TreeSpec {
    /// The seed of the generator.
    pub seed: u64,
    /// The folders under the root. Each has a README.md.
    pub folders: usize,
    /// The `.rs` files in each folder. Each has a `//!` header.
    pub files_per_folder: usize,
    /// The entries of the root's CASES.md, with the ids `GEN-000001` upward.
    pub cases: usize,
}

/// Writes the tree of `spec` under `root`, which must exist: a README.md and a CASES.md with `spec.cases` entries at the
/// root, then `spec.folders` folders, each with a README.md and `spec.files_per_folder` `.rs` files whose headers cite
/// case ids. Panics if a file cannot be written.
pub fn write_tree(root: &Path, spec: &TreeSpec) {
    let mut rng = Prng::new(spec.seed);
    let readme = format!("# Generated tree\n\n{}\n", sentence(&mut rng, 8));
    write_file(&root.join("README.md"), &readme);
    write_file(&root.join("CASES.md"), &cases_md(&mut rng, spec.cases));
    for folder in 0..spec.folders {
        let dir = root.join(format!("dir{folder:05}"));
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot make {}: {e}", dir.display()));
        let readme = format!("# Folder {folder:05}\n\n{}\n", sentence(&mut rng, 10));
        write_file(&dir.join("README.md"), &readme);
        for file in 0..spec.files_per_folder {
            let index = folder * spec.files_per_folder + file;
            let cite = (index < spec.cases).then(|| case_id(index));
            let source = source_text(&mut rng, cite);
            write_file(&dir.join(format!("file{file:05}.rs")), &source);
        }
    }
}

/// The id of the `index`th generated case, counting from zero: `GEN-000001` for index 0.
fn case_id(index: usize) -> String {
    format!("GEN-{:06}", index + 1)
}

/// The text of one generated `.rs` file: a `//!` header whose purpose sentence cites `cite` when there is one.
fn source_text(rng: &mut Prng, cite: Option<String>) -> String {
    let purpose = sentence(rng, 6);
    let header = match cite {
        Some(id) => format!("//! {purpose} See {id}."),
        None => format!("//! {purpose}"),
    };
    format!("{header}\n\npub const VALUE: u64 = {};\n", rng.next_u64())
}

/// The CASES.md of `cases` entries, opened by its `memex: cases` frontmatter: a `## Group` heading per `GROUP` entries, and each entry a backticked id and a claim.
fn cases_md(rng: &mut Prng, cases: usize) -> String {
    let mut text = String::from("---\nmemex: cases\n---\n# Generated cases\n");
    for index in 0..cases {
        if index % GROUP == 0 {
            text.push_str(&format!("\n## Group {}\n\n", index / GROUP + 1));
        }
        text.push_str(&format!("- `{}` {}\n", case_id(index), sentence(rng, 8)));
    }
    text
}

/// A sentence of `count` random words from `VOCAB`, with a capital first letter and a full stop. `count` is at least 1.
fn sentence(rng: &mut Prng, count: usize) -> String {
    let words: Vec<&str> = (0..count).map(|_| VOCAB[rng.below(VOCAB.len())]).collect();
    let mut text = words.join(" ");
    if let Some(first) = text.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    text.push('.');
    text
}

/// Writes `text` to `path`, replacing any file there. Panics with the path when the write fails.
fn write_file(path: &Path, text: &str) {
    fs::write(path, text).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}

/// A splitmix64 generator: a few lines, the same stream on every platform, and no crate.
pub struct Prng {
    state: u64,
}

impl Prng {
    /// A generator that starts from `seed`.
    pub fn new(seed: u64) -> Prng {
        Prng { state: seed }
    }

    /// The next 64 bits of the stream.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`. `n` must be at least 1.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}
