//! Name tests: memex on path names that hold a control character, run by the stress suite.
//! Cases: none (stress)

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::Path;

#[cfg(unix)]
use crate::support::{cmd, output_text, Sandbox};

/// The README of the root. Its sentence holds `names`, the word that `search names` looks for.
#[cfg(unix)]
const ROOT_README: &str = "# Names\n\nThe root readme holds the names.\n";

/// The memex commands that the test runs, each with its arguments: the overview, a search and the root's show.
#[cfg(unix)]
const VERBS: [&[&str]; 3] = [&["info"], &["search", "names"], &["show", "."]];

/// Path names with a control character are skipped, as names that are not UTF-8 are (design §4.4), so they never reach a
/// label. A tree with a file named `a<LF>b.rs` and a folder named `tab<TAB>here` gives the same `info`, `search names` and
/// `show .` as the same tree without them, each exit 0, and no control character in their output.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
#[cfg(unix)]
fn names_control_chars_skipped() {
    let sandbox = Sandbox::new();
    write_file(&sandbox.path().join("README.md"), ROOT_README);
    let without = run_verbs(&sandbox);
    write_control_names(sandbox.path());
    let with = run_verbs(&sandbox);
    assert_eq!(
        with, without,
        "a name with a control character changed the output"
    );
    for text in &with {
        assert_no_control(text);
    }
}

/// Windows cannot hold a control character in a file name, so there is no such tree to skip on that OS: the test is empty.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
#[cfg(not(unix))]
fn names_control_chars_skipped() {}

/// A folder whose name holds a control character is never entered, so one that cannot be listed gives no warning: `info`,
/// `search names` and `show .` each exit 0 with empty stderr (design §4.2 and §4.4). The folder is made unlistable (mode 000)
/// for the run, then restored, so the sandbox can be removed.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
#[cfg(unix)]
fn names_unlistable_control_folder_is_silent() {
    let sandbox = Sandbox::new();
    write_file(&sandbox.path().join("README.md"), ROOT_README);
    let folder = sandbox.path().join("esc\u{1b}");
    fs::create_dir(&folder).unwrap_or_else(|e| panic!("cannot make {}: {e}", folder.display()));
    write_file(
        &folder.join("README.md"),
        "# Escaped\n\nA folder whose name holds an escape.\n",
    );
    set_mode(&folder, 0o000);
    let runs = run_stderr(&sandbox);
    set_mode(&folder, 0o755);
    for (args, success, stderr) in runs {
        assert!(success, "memex {args} failed");
        assert!(stderr.is_empty(), "memex {args} wrote to stderr: {stderr}");
    }
}

/// Windows cannot hold a control character in a file name, so there is no such folder to skip on that OS: the test is empty.
#[test]
#[ignore = "stress suite: run with cargo test --release --test stress -- --ignored"]
#[cfg(not(unix))]
fn names_unlistable_control_folder_is_silent() {}

/// Writes the two names that hold a control character, each with a header that `search names` would find if it were
/// walked: a file `a<LF>b.rs`, and a folder `tab<TAB>here` with a README. Panics unless both are on disk.
#[cfg(unix)]
fn write_control_names(root: &Path) {
    write_file(
        &root.join("a\nb.rs"),
        "//! A file whose name holds a newline.\n\npub const VALUE: u64 = 1;\n",
    );
    let folder = root.join("tab\there");
    fs::create_dir(&folder).unwrap_or_else(|e| panic!("cannot make {}: {e}", folder.display()));
    write_file(
        &folder.join("README.md"),
        "# Tab folder\n\nA folder whose name has a tab.\n",
    );
    let names = listed(root);
    assert!(
        names.iter().any(|name| name == "a\nb.rs") && names.iter().any(|name| name == "tab\there"),
        "the names with control characters are not on disk: {names:?}"
    );
}

/// The output of each command in `VERBS`, run in the sandbox. Each must exit 0, and its stdout is returned as text.
#[cfg(unix)]
fn run_verbs(sandbox: &Sandbox) -> Vec<String> {
    VERBS
        .iter()
        .map(|args| {
            let out = cmd(sandbox)
                .args(*args)
                .output()
                .unwrap_or_else(|e| panic!("memex {args:?} did not start: {e}"));
            assert!(
                out.status.success(),
                "memex {args:?} failed: {}",
                output_text(&out.stderr)
            );
            output_text(&out.stdout)
        })
        .collect()
}

/// Runs each command of `VERBS` in the sandbox, and gives each one's arguments, whether it exited 0, and its stderr as text.
/// Nothing is asserted here, so a caller can restore the tree before it checks the results.
#[cfg(unix)]
fn run_stderr(sandbox: &Sandbox) -> Vec<(String, bool, String)> {
    VERBS
        .iter()
        .map(|args| {
            let out = cmd(sandbox)
                .args(*args)
                .output()
                .unwrap_or_else(|e| panic!("memex {args:?} did not start: {e}"));
            (
                format!("{args:?}"),
                out.status.success(),
                output_text(&out.stderr),
            )
        })
        .collect()
}

/// Sets the unix mode of `path`. Panics with the path when the change fails.
#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .unwrap_or_else(|e| panic!("cannot set the mode of {}: {e}", path.display()));
}

/// Asserts that `text` has no control character, except the line breaks that end its lines.
#[cfg(unix)]
fn assert_no_control(text: &str) {
    let found: Vec<char> = text
        .chars()
        .filter(|c| c.is_control() && *c != '\n')
        .collect();
    assert!(
        found.is_empty(),
        "control characters {found:?} in the output:\n{text}"
    );
}

/// The names in `dir`, as its listing gives them. Panics if the folder cannot be listed.
#[cfg(unix)]
fn listed(dir: &Path) -> Vec<String> {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()));
    entries
        .map(|entry| {
            let name = entry
                .unwrap_or_else(|e| panic!("a folder entry is unreadable: {e}"))
                .file_name();
            name.to_string_lossy().into_owned()
        })
        .collect()
}

/// Writes `text` to `path`, replacing any file there. Panics with the path when the write fails.
#[cfg(unix)]
fn write_file(path: &Path, text: &str) {
    fs::write(path, text).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}
