//! Scale tests of the log store and of single large files: 50k entries, a 1 MB line and 50 MB inputs.
//! Cases: none (stress)

use std::fs;
use std::time::Instant;

use super::scale::{
    fastest_info, intent, json_of, log_total, make_dir, note, over_budget, put, time_verb,
    write_in, MIB,
};
use super::support::{cmd, median_ms, output_text, report, run_stdin, Prng, Sandbox, NOW};

/// The log streams of the scale store, and the entries in each: 50k in all.
const STREAMS: usize = 10;
const PER_STREAM: usize = 5_000;

/// The body of each entry that `log_at` writes.
const LOG_BODY: &str = "# Scale entry\n\nA body of one log entry.\n";

/// A time far past the generated entries, so that `keep_logs` 3d expires all of them.
const LATER: &str = "2026-10-20T12:00:00Z";

/// The words of the generated long lines.
const WORDS: [&str; 6] = ["cache", "quiet", "stamp", "window", "trace", "yield"];

/// The claim of every case in the 50 MB CASES.md.
const CLAIM: &str = "Cache words hold a claim about the trace and the window of each walk, while the stamp of every marker stays quiet and the parse of a dense header keeps its token in the index, so the render of the root cites each claim once, and the cache stays quiet.";

/// A line of about `bytes` bytes of words, starting with the word `lineword`.
fn long_line(bytes: usize) -> String {
    let mut rng = Prng::new(4);
    let mut line = String::from("lineword");
    while line.len() < bytes {
        line.push(' ');
        line.push_str(WORDS[rng.below(WORDS.len())]);
    }
    line
}

/// A one-line README of about 1 MB and a README of 400k `](`: `info`, `search`, `show` of each and `show .` each finish in 1 s.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_long_lines() {
    let sandbox = Sandbox::new();
    write_in(
        &sandbox,
        "long/README.md",
        format!("# Long line\n\n{}\n", long_line(MIB)),
    );
    write_in(
        &sandbox,
        "links/README.md",
        format!("# Links\n\n{}\n", "](".repeat(400_000)),
    );
    let verbs: [&[&str]; 5] = [
        &["info"],
        &["search", "lineword"],
        &["show", "long/README.md"],
        &["show", "links/README.md"],
        &["show", "."],
    ];
    let mut breaches = Vec::new();
    for verb in verbs {
        let label = verb.join(" ");
        let ms = time_verb(&sandbox, &label, verb);
        breaches.extend(over_budget(&[(label.as_str(), ms, 1_000.0)]));
    }
    assert!(breaches.is_empty(), "over budget: {breaches:?}");
}

/// The layout that the 50 MB tests start from: a folder with a README and one small `.rs` file.
fn small_layout(sandbox: &Sandbox) {
    write_in(sandbox, "small/README.md", "# Small\n\nOne folder.\n");
    write_in(
        sandbox,
        "small/one.rs",
        "//! One small file.\n\npub const V: u64 = 1;\n",
    );
}

/// A 50 MB `.rs` file adds at most 50 ms to `info`: the fastest of five runs with it, against the fastest without it.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_50mb_rs_file_adds_50ms_to_info() {
    let without = Sandbox::new();
    small_layout(&without);
    let with = Sandbox::new();
    small_layout(&with);
    let huge = "pub const VALUE: u64 = 1;\n".repeat((50 * MIB).div_ceil(26));
    assert!(
        huge.len() >= 50 * MIB,
        "the generated file is {} bytes",
        huge.len()
    );
    write_in(
        &with,
        "small/huge.rs",
        format!("//! Huge generated source.\n\n{huge}"),
    );
    let base = fastest_info(&without);
    let grown = fastest_info(&with);
    let added = grown - base;
    report("info, no 50 MB file, fastest of 5", base);
    report("info, with a 50 MB .rs file, fastest of 5", grown);
    note(&format!(
        "info adds {added:.1} ms for the 50 MB file (budget 50 ms)"
    ));
    assert!(
        added <= 50.0,
        "the 50 MB file adds {added:.1} ms to info, the budget is 50 ms"
    );
}

/// A 50 MB README: `show` of it prints within 3 s.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_50mb_readme_show() {
    let sandbox = Sandbox::new();
    let text = "Cache words mark a line of the big README.\n".repeat((50 * MIB).div_ceil(43));
    assert!(text.len() >= 50 * MIB, "the README is {} bytes", text.len());
    write_in(&sandbox, "big/README.md", text);
    let ms = time_verb(
        &sandbox,
        "show big/README.md, 50 MB",
        &["show", "big/README.md"],
    );
    assert!(ms <= 3_000.0, "show took {ms:.1} ms, the budget is 3000 ms");
}

/// A 50 MB CASES.md of 200k cases: `info` counts them all and runs within 3 s.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_50mb_cases_info() {
    let mut text = String::from("---\nmemex: cases\n---\n# Big cases\n");
    for index in 0..200_000 {
        if index % 100 == 0 {
            text.push_str(&format!("\n## Group {}\n\n", index / 100 + 1));
        }
        text.push_str(&format!("- `GEN-{:06}` {CLAIM}\n", index + 1));
    }
    assert!(
        text.len() >= 50 * MIB,
        "the CASES.md is {} bytes",
        text.len()
    );
    note(&format!("CASES.md is {} bytes", text.len()));
    let sandbox = Sandbox::new();
    write_in(&sandbox, "CASES.md", text);
    let info = json_of(&sandbox, &["info", "--json"]);
    assert_eq!(intent(&info)[2], 200_000, "the cases of the 50 MB CASES.md");
    let ms = time_verb(&sandbox, "info, 50 MB CASES.md, 200k cases", &["info"]);
    assert!(ms <= 3_000.0, "info took {ms:.1} ms, the budget is 3000 ms");
}

/// A sandbox with `memex init` run in it, as a project starts.
fn init_sandbox() -> Sandbox {
    let sandbox = Sandbox::new();
    let init = sandbox.memex_init();
    assert!(
        init.status.success(),
        "memex init failed: {}",
        output_text(&init.stderr)
    );
    sandbox
}

/// Replaces the store's entries with 50k generated ones in ten streams, stamped one second apart from 2026-10-01 00:00:00 UTC,
/// and sets `keep_logs` to `keep`.
fn write_store(sandbox: &Sandbox, keep: &str) {
    let root = sandbox.path();
    let store = root.join(".memex/logs");
    // A first write has no store yet, so the error of removing it is expected and ignored.
    let _ = fs::remove_dir_all(&store);
    put(
        &root.join(".memex/config.json"),
        format!("{{\"keep_logs\": \"{keep}\"}}"),
    );
    for stream in 0..STREAMS {
        let name = format!("stream-{stream:02}");
        let dir = store.join(&name);
        make_dir(&dir);
        for index in 0..PER_STREAM {
            let (hh, mm, ss) = (index / 3600, (index / 60) % 60, index % 60);
            let body = format!(
                "---\nstream: {name}\ntime: 2026-10-01T{hh:02}:{mm:02}:{ss:02}Z\n---\n# Scale entry {index}\n\nGenerated for the scale test.\n"
            );
            let file = format!("20261001-{hh:02}{mm:02}{ss:02}-scale-{index:05}.md");
            put(&dir.join(file), body);
        }
    }
}

/// Runs `memex log stream-00` with `LOG_BODY` on stdin and `MEMEX_NOW` set to `now`. Panics unless it exits 0.
fn log_at(sandbox: &Sandbox, now: &str) {
    let logged = run_stdin(
        cmd(sandbox)
            .args(["log", "stream-00"])
            .env("MEMEX_NOW", now),
        LOG_BODY,
    );
    assert!(
        logged.status.success(),
        "memex log failed: {}",
        output_text(&logged.stderr)
    );
}

/// 50k log entries with `keep_logs` forever: `info` counts them all within 1.5 s, and one `memex log` runs within 0.5 s.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_50k_log_entries_info_and_log() {
    let sandbox = init_sandbox();
    write_store(&sandbox, "forever");
    let info = json_of(&sandbox, &["info", "--json"]);
    assert_eq!(log_total(&info), 50_000, "the entries that info counts");
    let info_ms = time_verb(&sandbox, "info, 50k log entries", &["info"]);
    let log_ms = median_ms(3, 1, || log_at(&sandbox, NOW));
    report("memex log, 50k entries, keep_logs forever", log_ms);
    let timed = [("info", info_ms, 1_500.0), ("memex log", log_ms, 500.0)];
    let breaches = over_budget(&timed);
    assert!(breaches.is_empty(), "over budget: {breaches:?}");
}

/// A prune of all 50k expired entries (`keep_logs` 3d, `MEMEX_NOW` ten days on): one `memex log` runs within 5 s, by the median
/// of three runs, and leaves one entry.
#[test]
#[ignore = "stress suite: cargo test --release --test stress -- --ignored scale_"]
fn scale_50k_log_prune_within_5s() {
    let sandbox = init_sandbox();
    let mut times = Vec::new();
    for round in 0..3 {
        write_store(&sandbox, "3d");
        let start = Instant::now();
        log_at(&sandbox, LATER);
        times.push(start.elapsed().as_secs_f64() * 1_000.0);
        let info = json_of(&sandbox, &["info", "--json"]);
        assert_eq!(
            log_total(&info),
            1,
            "round {round}: the prune leaves one entry"
        );
    }
    times.sort_by(f64::total_cmp);
    let ms = times[times.len() / 2];
    report("memex log that prunes 50k entries, median of 3", ms);
    assert!(
        ms <= 5_000.0,
        "the prune took {ms:.1} ms, the budget is 5000 ms"
    );
}
