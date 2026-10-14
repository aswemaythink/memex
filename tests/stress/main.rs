//! Entry point of the stress suite: opt-in tests of the built memex, run with `--ignored`.
//!
//! The modules are `scale` (large intent layers), `concurrency` (many writers on one store), `fuzz` with `fuzz_parse` (hostile input),
//! `names` (path names with control characters) and `budget` (timing budgets on a real repository). Each family has its
//! own module, and `support` is their shared code.
//! Cases: none (stress)

#[allow(dead_code)]
mod support;

mod budget;
mod conc_worktrees;
mod concurrency;
mod fuzz;
mod fuzz_parse;
mod names;
mod scale;
mod scale_logs;

use support::{cmd, output_text, run_stdin, Sandbox};

/// The body the smoke test logs. Its title makes the slug of the ref, and its marker word is what `search` must find.
const BODY: &str = "# Stress smoke\n\nThe quasarbeacon marks the round trip.\n";

/// `log` stores a body, `show` prints it back and `search` finds it: the round trip works with MEMEX_NOW fixed.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
fn smoke_round_trip() {
    let sandbox = Sandbox::new();
    let git = sandbox.git_init();
    assert!(
        git.status.success(),
        "git init failed: {}",
        output_text(&git.stderr)
    );
    let init = sandbox.memex_init();
    assert!(
        init.status.success(),
        "memex init failed: {}",
        output_text(&init.stderr)
    );

    let logged = run_stdin(cmd(&sandbox).arg("log").arg("impl"), BODY);
    assert!(
        logged.status.success(),
        "memex log failed: {}",
        output_text(&logged.stderr)
    );
    let reference = output_text(&logged.stdout).trim_end().to_string();
    assert_eq!(reference, "impl:20261014-120000-stress-smoke");

    let shown = cmd(&sandbox)
        .arg("show")
        .arg(&reference)
        .output()
        .expect("memex show runs");
    assert!(
        shown.status.success(),
        "memex show failed: {}",
        output_text(&shown.stderr)
    );
    assert!(output_text(&shown.stdout).contains("The quasarbeacon marks the round trip."));

    let found = cmd(&sandbox)
        .arg("search")
        .arg("quasarbeacon")
        .output()
        .expect("memex search runs");
    assert!(
        found.status.success(),
        "memex search failed: {}",
        output_text(&found.stderr)
    );
    assert!(
        output_text(&found.stdout).contains(&reference),
        "search did not find {reference}"
    );
}
