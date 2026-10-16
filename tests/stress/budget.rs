//! Budget tests: the read verbs against their timing budgets, run by the stress suite.
//! `budget_info`, `budget_search`, `budget_show` and `budget_check` time `memex info`, `memex search button`, `memex show packages`
//! and `memex check` in the repository that MEMEX_STRESS_REPO names: a release build, three untimed warm-ups, then the median of
//! nine timed runs against 90, 115, 95 and 150 ms. Without the variable each test prints a skip line and passes. The commands only read that
//! repository; they run there with MEMEX_* removed, so the real clock is read, and with HOME unchanged.
//! Cases: none (stress)

use std::env;
use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Instant;

use crate::support::{bin, load_avg, output_text};

/// The variable that names the reference repository the budgets run in.
const REPO_VAR: &str = "MEMEX_STRESS_REPO";

/// The untimed runs before the timed ones, so that the binary and the page cache are warm.
const WARM_UP: usize = 3;

/// The timed runs of each command. Their median is the number that the budget judges.
const RUNS: usize = 9;

/// `memex info` in the reference repository stays within its 90 ms budget.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn budget_info() {
    check_budget("budget_info", &["info"], 90.0);
}

/// `memex search button` in the reference repository stays within its 115 ms budget.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn budget_search() {
    check_budget("budget_search", &["search", "button"], 115.0);
}

/// `memex show packages` in the reference repository stays within its 95 ms budget.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn budget_show() {
    check_budget("budget_show", &["show", "packages"], 95.0);
}

/// `memex check` in the reference repository stays within its 150 ms budget.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn budget_check() {
    check_budget("budget_check", &["check"], 150.0);
}

/// Times `memex <args>` in the repository that MEMEX_STRESS_REPO names and asserts that its median is within `budget_ms`.
/// It prints the median, fastest, slowest and load average, and every warning that makes the timing not mean anything.
fn check_budget(name: &str, args: &[&str], budget_ms: f64) {
    let Some(repo) = env::var_os(REPO_VAR).filter(|value| !value.is_empty()) else {
        say(&format!("{name}: skipped: set {REPO_VAR}"));
        return;
    };
    let repo = PathBuf::from(repo);
    assert!(
        repo.is_dir(),
        "{REPO_VAR} is {}, which is not a folder",
        repo.display()
    );
    let timing = Timing::of(time_runs(&repo, args));
    let load = load_avg();
    let warnings = load_warnings(&load);
    say(&format!(
        "{name}: memex {}: median {:.1} ms, min {:.1} ms, max {:.1} ms over {RUNS} runs after {WARM_UP} warm-ups; budget {budget_ms:.0} ms; load {load}",
        args.join(" "),
        timing.median,
        timing.min,
        timing.max,
    ));
    say(&format!("{name}: runs (ms): {}", timing.runs_text()));
    for warning in &warnings {
        say(&format!("!!! {warning} !!!"));
    }
    let note: String = warnings
        .iter()
        .map(|warning| format!("; {warning}"))
        .collect();
    assert!(
        timing.median <= budget_ms,
        "{name}: median {:.1} ms is over the {budget_ms:.0} ms budget (min {:.1} ms, max {:.1} ms){note}",
        timing.median,
        timing.min,
        timing.max,
    );
}

/// The timed runs of one command, in run order, with their median, fastest and slowest, in milliseconds.
struct Timing {
    runs: Vec<f64>,
    median: f64,
    min: f64,
    max: f64,
}

impl Timing {
    /// Sorts a copy of `runs` to find the median (the upper middle for an even count), the fastest and the slowest.
    /// `runs` is never empty: `time_runs` always returns `RUNS` runs.
    fn of(runs: Vec<f64>) -> Timing {
        let mut sorted = runs.clone();
        sorted.sort_by(f64::total_cmp);
        let last = sorted.len() - 1;
        Timing {
            median: sorted[sorted.len() / 2],
            min: sorted[0],
            max: sorted[last],
            runs,
        }
    }

    /// The timed runs as text, in run order, such as `93.4 90.1 ...`.
    fn runs_text(&self) -> String {
        self.runs
            .iter()
            .map(|ms| format!("{ms:.1}"))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

/// Runs `memex <args>` in `repo` `WARM_UP` times untimed, then `RUNS` times timed, and returns the timed wall times in ms.
fn time_runs(repo: &Path, args: &[&str]) -> Vec<f64> {
    for _ in 0..WARM_UP {
        run_timed(repo, args);
    }
    (0..RUNS).map(|_| run_timed(repo, args)).collect()
}

/// Runs `memex <args>` in `repo` and returns its wall time in milliseconds, spawn included, as the bench one-liner measures
/// it. Its stdout goes nowhere. A run that does not start, or exits non-zero, panics with its stderr.
fn run_timed(repo: &Path, args: &[&str]) -> f64 {
    let mut command = memex_in(repo, args);
    let start = Instant::now();
    let output = command
        .output()
        .unwrap_or_else(|e| panic!("memex {} did not start: {e}", args.join(" ")));
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    assert!(
        output.status.success(),
        "memex {} exited with {}: {}",
        args.join(" "),
        output.status,
        output_text(&output.stderr)
    );
    ms
}

/// A `memex <args>` command whose cwd is `repo`. Its MEMEX_* variables are removed, so that the real clock is read and no
/// fixed time or agent leaks in. HOME and the rest of the environment are inherited, as a user's own run would have them.
fn memex_in(repo: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(bin());
    command
        .args(args)
        .current_dir(repo)
        .stdin(Stdio::null())
        .stdout(Stdio::null());
    for (name, _) in env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("MEMEX_")
        {
            command.env_remove(&name);
        }
    }
    command
}

/// The warnings that make a timing not mean anything: a debug build, and a 1-minute load above half the CPUs. `load` is
/// what `load_avg` printed, and an unknown load (`unknown`) raises no load warning.
fn load_warnings(load: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    if cfg!(debug_assertions) {
        warnings
            .push("DEBUG BUILD, timings not meaningful: run with cargo test --release".to_string());
    }
    let cpus = thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let limit = cpus as f64 / 2.0;
    let one_minute: Option<f64> = load
        .split_whitespace()
        .next()
        .and_then(|field| field.parse().ok());
    if let Some(one_minute) = one_minute.filter(|value| *value > limit) {
        warnings.push(format!(
            "LOAD TOO HIGH, timings not meaningful: 1-min load {one_minute:.2} > {limit:.1} (half of {cpus} CPUs)"
        ));
    }
    warnings
}

/// Writes one line to stderr directly, so that it shows on a passing run without `--nocapture`, as support's `report` does.
fn say(line: &str) {
    let _ = writeln!(io::stderr(), "{line}");
}
