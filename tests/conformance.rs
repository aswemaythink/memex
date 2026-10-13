//! Entry point of the conformance suite: the six conformity rules first, then one trial per case id.
//! Cases: all (runner)

#[allow(dead_code)] // this binary uses part of the shared harness; runner_selftest uses the rest
mod harness;

use std::path::{Path, PathBuf};

use libtest_mimic::{Arguments, Failed, Trial};

use harness::bless;
use harness::conform;
use harness::discover::{self, CaseRef};
use harness::sandbox::Sandbox;
use harness::transcript;

fn main() {
    let mut args = Arguments::from_args();
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join("tests").join("cases");
    let violations = conform::check(&root);

    let mut trials = Vec::new();
    for rule in 1..=6 {
        let tag = format!("rule{rule}:");
        let failing: Vec<String> = violations
            .iter()
            .filter(|line| line.starts_with(&tag))
            .cloned()
            .collect();
        // While any rule fails, only the failing rules are reported.
        if violations.is_empty() || !failing.is_empty() {
            trials.push(Trial::test(format!("conformity::rule{rule}"), move || {
                if failing.is_empty() {
                    Ok(())
                } else {
                    Err(Failed::from(failing.join("\n")))
                }
            }));
        }
    }

    if violations.is_empty() {
        for case in discover::cases(&root) {
            let ignored = harness::load_spec(&case.dir.join("case.json"))
                .map(|spec| !spec.runs_on_this_os())
                .unwrap_or(false);
            let name = case.id.clone();
            trials.push(
                Trial::test(name, move || run_case(&case, manifest)).with_ignored_flag(ignored),
            );
        }
    } else {
        // A name filter would hide the failing rules and leave nothing to run, which would pass.
        args.filter = None;
        args.skip.clear();
        args.ignored = false;
    }

    libtest_mimic::run(&args, trials).exit();
}

/// Runs one case in a fresh sandbox and judges its transcript against expect.txt.
fn run_case(case: &CaseRef, manifest: &Path) -> Result<(), Failed> {
    let spec = harness::load_spec(&case.dir.join("case.json"))
        .map_err(|e| Failed::from(format!("case.json {e}")))?;
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_memex"));
    let sandbox = Sandbox::create(&case.dir, &spec, &bin).map_err(Failed::from)?;
    let mut steps = Vec::new();
    for step in &spec.steps {
        steps.push(sandbox.run_step(step, &case.dir).map_err(Failed::from)?);
    }
    let rendered =
        transcript::render(&steps, &sandbox.root, &spec.snapshot).map_err(Failed::from)?;
    let spellings = transcript::sandbox_spellings(&sandbox.root);
    let text = transcript::redact(&rendered, &spellings, env!("CARGO_PKG_VERSION"));
    let expect = case.dir.join("expect.txt");
    let shown = display_path(&expect, manifest);
    bless::judge(&case.id, &expect, &text, &shown).map_err(Failed::from)
}

/// The path relative to the crate root, for messages.
fn display_path(path: &Path, manifest: &Path) -> String {
    path.strip_prefix(manifest)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
