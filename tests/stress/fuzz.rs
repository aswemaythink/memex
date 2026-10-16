//! Fuzz tests: memex on generated and hostile input, run by the stress suite.
//! Each property runs the built binary on inputs from `fuzz_parse` and from the README and argv generators here, with a fixed
//! seed, so a failure reproduces and shrinks to the minimal input that the panic message prints.
//! Cases: none (stress)

use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Output, Stdio};
use std::time::Instant;

use proptest::collection::vec;
use proptest::prelude::{prop_oneof, Strategy};
use proptest::sample::select;
use proptest::test_runner::{
    Config, RngAlgorithm, TestCaseError, TestCaseResult, TestRng, TestRunner,
};

use super::fuzz_parse::{self, agent_fields, roster_ids, CasesDoc, LogBody, SourceFile};
use super::support::{cmd, min_ms, output_text, report, run_stdin, write_tree, Sandbox, TreeSpec};

/// The seed of every property: 32 bytes, as ChaCha takes them, so each run draws the same inputs.
const SEED: [u8; 32] = *b"memex-stress-fuzz-seed-v1-202610";

/// More than a pipe holds: a roster this large must block on a pipe that the reader has closed.
const PIPE_OVERFLOW: usize = 256 * 1024;

/// The most milliseconds that `show .` may take, on a generated README and on the 400k-link README.
const LINK_BUDGET_MS: f64 = 1000.0;

/// The tree of the argv property: a few folders, files and cases.
const ARGV_TREE: TreeSpec = TreeSpec {
    seed: 7,
    folders: 3,
    files_per_folder: 4,
    cases: 12,
};

/// The tree of the closed-stdout test: 4000 header files, so the roster is far larger than a pipe.
const WIDE_TREE: TreeSpec = TreeSpec {
    seed: 11,
    folders: 100,
    files_per_folder: 40,
    cases: 50,
};

/// Words that a citation is made of, with the markup around them: backticks, `](`, ids, paths and fences.
#[rustfmt::skip]
const CITATIONS: &[&str] = &[
    "`", "``", "```", "~~~", "](", "](src/x.rs)", "[t](../y.md)", "[t](https://e.com/x)", "[a](#frag)", "GEN-0001",
    "ALPHA-BETA-002", "NAME-*", "`src/a.rs`", "`GEN-0001`", "cases:GEN-0001", "impl:20261014-120000-x", "CASES.md",
    "src/cli.rs", "../x", "/abs/path.md", "x.y.z", "UTF-8", "é", "🙂", ")", "[", "]", "(", "- item", "# hash",
    "  indent", "`](x`", "*", "_", "\n```\n", "\n~~~\n", "\n```rust\n",
];

/// Verbs, flags and values of design §3, and junk, as the words of an argument vector.
#[rustfmt::skip]
const ATOMS: &[&str] = &[
    "search", "show", "log", "info", "init", "--help", "--version", "--", "-", "", "frob", "SEARCH", "--in", "--limit",
    "--up", "--json", "--to", "--tag", "--agent", "--in=cases", "--limit=3", "--json=1", "--unknown", "-x", "--limit=-1",
    "--limit=abc", "--up=99999999999999999999999", "--agent=cook-2", "--tag=x,y", "cases", "headers,readmes", "logs",
    "nope", "3", "0", "-1", "99999999999999999999999", "impl", "cook-2", "a,b", ",", "x y", "é", "\u{1b}[31m",
    "line\nbreak", "tab\there", "GEN-0001", "NAME-*", "cases:GEN-0001", ".", "..", "../x", "/", "src/x.rs",
    "impl:20261014-120000-title", "to:cook-2", "tag:x", "since:2d", "since:bogus", "agent:x", "branch:main",
    "worktree:x", "std::io", "x:", "::", "\\", "*", "dir00000", "dir00000/file00001.rs", "README.md", "CASES.md",
];

#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored fuzz_"]
fn fuzz_log_frontmatter() {
    check(fuzz_parse::log_body(), |body: LogBody| {
        let sandbox = Sandbox::new();
        run_ok(&sandbox, &words(&["init"]), "")?;
        let log = words(&["log", "impl"]);
        let Some(expected) = body.fields else {
            let refused = run(&sandbox, &log, &body.text);
            return match refused.status.code() {
                Some(2) => Ok(()),
                _ => fail(describe(&log, &refused)),
            };
        };
        let logged = run_ok(&sandbox, &log, &body.text)?;
        let reference = output_text(&logged.stdout).trim_end().to_string();
        let shown = run_ok(
            &sandbox,
            &words(&["show", reference.as_str(), "--json"]),
            "",
        )?;
        match agent_fields(&shown.stdout) {
            Ok(fields) if fields == expected => Ok(()),
            Ok(fields) => fail(format!(
                "show printed {fields:?}, the body wrote {expected:?}: {:?}",
                body.text
            )),
            Err(problem) => fail(problem),
        }
    });
}

#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored fuzz_"]
fn fuzz_cases_round_trip() {
    check(fuzz_parse::cases_doc(), |doc: CasesDoc| {
        let sandbox = Sandbox::new();
        write(&sandbox, "CASES.md", doc.text.as_bytes());
        let roster = words(&["search", "--in", "cases", "--limit", "100000"]);
        let printed = roster_ids(&output_text(&run_ok(&sandbox, &roster, "")?.stdout));
        let (mut seen, mut declared) = (printed.clone(), doc.declared);
        if declared.is_empty() {
            // design §2.2: a collection file with no id-led items is one doc, its id the file stem.
            declared.push("CASES".to_string());
        }
        seen.sort();
        declared.sort();
        if seen != declared {
            return fail(format!(
                "search printed {seen:?}, the file declares {declared:?}"
            ));
        }
        for id in &printed {
            let label = format!("=== cases:{id} ===");
            let shown = run_ok(&sandbox, &words(&["show", id.as_str()]), "")?;
            if !output_text(&shown.stdout).contains(label.as_str()) {
                return fail(format!("show {id} did not print {label}"));
            }
        }
        Ok(())
    });
}

#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored fuzz_"]
fn fuzz_header_bytes() {
    check(fuzz_parse::source_files(), |files: Vec<SourceFile>| {
        let sandbox = Sandbox::new();
        for file in &files {
            write(&sandbox, &file.name, &file.bytes);
        }
        let mut runs = vec![
            words(&["init"]),
            words(&["info"]),
            words(&["info", "--json"]),
        ];
        runs.extend([
            words(&["search", "file"]),
            words(&["search", "--in", "headers"]),
        ]);
        runs.extend([words(&["show", "."]), words(&["show", "src"])]);
        for file in &files {
            runs.push(words(&["show", file.name.as_str()]));
            runs.push(words(&["show", file.name.as_str(), "--json"]));
        }
        for args in &runs {
            run_ok(&sandbox, args, "")?;
        }
        run_ok(
            &sandbox,
            &words(&["log", "impl"]),
            "# Header probe\n\nBody.\n",
        )
        .map(|_| ())
    });
}

#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored fuzz_"]
fn fuzz_readme_citations() {
    check(readme_text(), |text: String| {
        let sandbox = Sandbox::new();
        write(&sandbox, "README.md", text.as_bytes());
        let started = Instant::now();
        run_ok(&sandbox, &words(&["show", "."]), "")?;
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        if elapsed_ms > LINK_BUDGET_MS {
            return fail(format!(
                "show . took {elapsed_ms:.0} ms, the budget is {LINK_BUDGET_MS:.0} ms"
            ));
        }
        for args in [
            words(&["info"]),
            words(&["show", "README.md"]),
            words(&["search", "code"]),
        ] {
            run_ok(&sandbox, &args, "")?;
        }
        Ok(())
    });
}

#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored fuzz_"]
fn fuzz_argv() {
    check(argv(), |argv: Vec<String>| {
        let sandbox = Sandbox::new();
        write_tree(sandbox.path(), &ARGV_TREE);
        run_ok(&sandbox, &words(&["init"]), "")?;
        let mut with_json = argv.clone();
        with_json.push("--json".to_string());
        for args in [argv, with_json] {
            let output = run(&sandbox, &args, "# Argv probe\n\nBody.\n");
            let code = output.status.code();
            let stderr = output_text(&output.stderr);
            if !matches!(code, Some(0..=3)) || stderr.contains("panicked") {
                return fail(format!(
                    "exit {code:?}, stderr {stderr:?}, for memex {args:?}"
                ));
            }
        }
        Ok(())
    });
}

/// A README of 400k `](` on one line: `show .` exits 0, within the budget at its best of three runs.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored fuzz_"]
fn fuzz_readme_400k_links() {
    let sandbox = Sandbox::new();
    write(
        &sandbox,
        "README.md",
        format!("# Pathological\n\n{}\n", "](".repeat(400_000)).as_bytes(),
    );
    let show = words(&["show", "."]);
    let ms = min_ms(3, || {
        let shown = run(&sandbox, &show, "");
        assert!(shown.status.success(), "{}", describe(&show, &shown));
    });
    report("show . on a README of 400k `](` on one line", ms);
    assert!(
        ms <= LINK_BUDGET_MS,
        "show . took {ms:.0} ms at best, the budget is {LINK_BUDGET_MS:.0} ms"
    );
}

/// `search` into a pipe that the test closes after one line: memex exits 0 and says nothing on stderr (design §3.6).
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored fuzz_"]
fn fuzz_closed_stdout() {
    let sandbox = Sandbox::new();
    write_tree(sandbox.path(), &WIDE_TREE);
    let search = words(&["search", "--in", "headers", "--limit", "100000"]);
    let full = run(&sandbox, &search, "");
    let size = full.stdout.len();
    assert!(
        full.status.success() && size > PIPE_OVERFLOW,
        "{} ({size} bytes)",
        describe(&search, &full)
    );
    let spawned = cmd(&sandbox)
        .args(&search)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = spawned.unwrap_or_else(|e| panic!("memex did not start: {e}"));
    let stdout = child
        .stdout
        .take()
        .unwrap_or_else(|| panic!("the child has no stdout pipe"));
    let mut first = String::new();
    // A temporary reader: it is dropped at the end of this statement, which closes the pipe while the roster is unread.
    let read = BufReader::new(stdout).read_line(&mut first);
    assert!(
        matches!(read, Ok(n) if n > 0) && first.starts_with("headers"),
        "first line {first:?}, read {read:?}"
    );
    let closed = child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("memex did not finish: {e}"));
    assert!(closed.status.success(), "{}", describe(&search, &closed));
    assert!(
        closed.stderr.is_empty(),
        "a closed stdout printed: {}",
        output_text(&closed.stderr)
    );
}

/// A README: a title, then paragraphs of citation words and printable text, with fences that may stay open.
fn readme_text() -> impl Strategy<Value = String> {
    let token =
        prop_oneof![4 => select(CITATIONS.to_vec()).prop_map(String::from), 1 => "[ -~]{0,12}"];
    ("[A-Za-z][A-Za-z ]{0,12}", vec(vec(token, 0..=8), 0..=5)).prop_map(|(title, paragraphs)| {
        let body = paragraphs
            .iter()
            .map(|words| format!("{}\n\n", words.join(" ")));
        format!("# {title}\n\n{}", body.collect::<Vec<_>>().concat())
    })
}

/// Argument vectors of up to six words, drawn from the real grammar and from junk.
fn argv() -> impl Strategy<Value = Vec<String>> {
    let word = prop_oneof![5 => select(ATOMS.to_vec()).prop_map(String::from), 1 => "\\PC{0,12}"];
    vec(word, 0..=6)
}

/// Runs `test` on `cases()` values of `strategy`, drawn from SEED with no failure file. A failing value panics with the
/// minimal value that proptest shrank it to.
fn check<S: Strategy>(strategy: S, test: impl Fn(S::Value) -> TestCaseResult) {
    let cases = std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|value| value.parse().ok());
    let config = Config {
        cases: cases.unwrap_or(64),
        failure_persistence: None,
        ..Config::default()
    };
    let rng = TestRng::from_seed(RngAlgorithm::ChaCha, &SEED);
    if let Err(error) = TestRunner::new_with_rng(config, rng).run(&strategy, test) {
        panic!("{error}");
    }
}

/// Runs memex with `args` in the sandbox, with `input` on its stdin, and returns its output.
fn run(sandbox: &Sandbox, args: &[String], input: &str) -> Output {
    run_stdin(cmd(sandbox).args(args), input)
}

/// Runs memex as `run` does, and returns its output when it exits 0. Otherwise the property fails with the run's account.
fn run_ok(sandbox: &Sandbox, args: &[String], input: &str) -> Result<Output, TestCaseError> {
    let output = run(sandbox, args, input);
    if output.status.success() {
        Ok(output)
    } else {
        fail(describe(args, &output))
    }
}

/// A failed property, with the message that proptest prints beside the minimal input.
fn fail<T>(message: String) -> Result<T, TestCaseError> {
    Err(TestCaseError::fail(message))
}

/// One line about a run for a failure message: its words, its exit code and its stderr.
fn describe(args: &[String], output: &Output) -> String {
    let stderr = output_text(&output.stderr);
    format!(
        "memex {}: exit {:?}, stderr {stderr:?}",
        args.join(" "),
        output.status.code()
    )
}

/// The words of a command line.
fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| word.to_string()).collect()
}

/// Writes `bytes` to `name` under the sandbox, making its folders first. Panics when the write fails.
fn write(sandbox: &Sandbox, name: &str, bytes: &[u8]) {
    let path = sandbox.path().join(name);
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)
            .unwrap_or_else(|e| panic!("cannot make {}: {e}", folder.display()));
    }
    fs::write(&path, bytes).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}
