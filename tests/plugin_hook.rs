//! Runs the plugin's hook script on sample hook input and checks its output (design §6, PLAN S6.1).
//! Cases: none (the plugin hook is pinned here, not by a conformance case)
//!
//! These tests run on Unix only. On Windows this file compiles to no tests: they drive the POSIX sh hook with `sh`, Unix
//! symlinks and Unix file modes, and none of that is run there. The rest of the suite still runs on Windows.

#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde_json::Value;
use tempfile::TempDir;

/// The hook script, relative to the crate root.
const HOOK: &str = "plugins/memex/hooks/start.sh";

/// The PATH the hook runs under: sed, head, tr and cat are here, and memex is not (a test checks that).
const SYSTEM_PATH: &str = "/usr/bin:/bin";

/// The clock every run sees, so that `memex info` prints the same text each time.
const NOW: &str = "2026-10-14T12:00:00Z";

/// The most bytes of context that a SubagentStart hook gives (the hook's own cap, below the host's limit).
const CAP: usize = 9000;

/// The install hint, as the hook prints it without memex on PATH. SessionStart prints it with a newline after.
const HINT: &str = "memex not found: npx @rynwhite/memex init\nalternative: cargo install agent-memex, then memex init";

/// Serialises the spawns of this file. A fake memex written while another test forks can be refused with "text file busy".
static SPAWN: Mutex<()> = Mutex::new(());

#[test]
fn session_start_prints_the_info_overview() {
    let repo = fixture();
    let bin = memex_link();
    let hook = run_hook(&with_memex(&bin), &session_json(), repo.path());
    assert!(hook.status.success(), "the hook exits 0: {hook:?}");
    let info = run_memex(&["info"], repo.path());
    assert!(
        text(&info).starts_with("memex  "),
        "memex info prints its header: {:?}",
        text(&info)
    );
    assert_eq!(
        text(&hook),
        text(&info),
        "SessionStart prints exactly `memex info`"
    );
}

#[test]
fn subagent_start_gives_the_id_line_and_the_agent_info_as_json() {
    let repo = fixture();
    let bin = memex_link();
    let hook = run_hook(&with_memex(&bin), &subagent_json("cook"), repo.path());
    let info = run_memex(&["info", "--agent", "cook"], repo.path());
    assert_eq!(
        additional_context(&hook),
        format!("{}\n{}", id_line("cook"), text(&info))
    );
}

#[test]
fn subagent_start_escapes_quotes_backslashes_tabs_and_carriage_returns() {
    let repo = fixture();
    let bin = fake_memex(
        r##"printf 'say "hi" back\\slash\ttab\rcr\nnext\nagent=%s\n' "${MEMEX_AGENT:-unset}""##,
    );
    let hook = run_hook(&with_memex(&bin), &subagent_json("cook"), repo.path());
    let expected = format!(
        "{}\nsay \"hi\" back\\slash\ttab\rcr\nnext\nagent=unset\n",
        id_line("cook")
    );
    assert_eq!(additional_context(&hook), expected);
}

#[test]
fn subagent_start_caps_the_context_at_9000_bytes() {
    let repo = fixture();
    let bin = fake_memex(r##"head -c 10000 /dev/zero | tr '\000' a"##);
    let hook = run_hook(&with_memex(&bin), &subagent_json("cook"), repo.path());
    let context = additional_context(&hook);
    assert_eq!(context.len(), CAP);
    assert!(context.starts_with(&id_line("cook")));
}

#[test]
fn subagent_start_falls_back_to_subagent_for_an_unsafe_or_missing_agent_type() {
    let repo = fixture();
    let bin = memex_link();
    let path = with_memex(&bin);
    let long = "a".repeat(65);
    let inputs = [
        r#"{"hook_event_name":"SubagentStart","agent_type":"co\"ok"}"#.to_string(),
        r#"{"hook_event_name":"SubagentStart","agent_type":"a b"}"#.to_string(),
        format!(r#"{{"hook_event_name":"SubagentStart","agent_type":"{long}"}}"#),
        r#"{"hook_event_name":"SubagentStart"}"#.to_string(),
    ];
    for input in inputs {
        let hook = run_hook(&path, &input, repo.path());
        assert!(
            additional_context(&hook).starts_with(&id_line("subagent")),
            "fallback for {input}"
        );
    }
}

#[test]
fn subagent_start_keeps_a_64_character_agent_type() {
    let repo = fixture();
    let bin = memex_link();
    let agent = "a".repeat(64);
    let hook = run_hook(&with_memex(&bin), &subagent_json(&agent), repo.path());
    assert!(additional_context(&hook).starts_with(&id_line(&agent)));
}

#[test]
fn without_memex_on_path_both_events_print_the_install_hint_and_exit_zero() {
    assert!(
        !Path::new("/usr/bin/memex").exists() && !Path::new("/bin/memex").exists(),
        "memex is installed in /usr/bin or /bin, so the hook would find it: this test needs it absent"
    );
    let repo = fixture();
    let session = run_hook(SYSTEM_PATH, &session_json(), repo.path());
    assert!(session.status.success(), "the hook exits 0: {session:?}");
    assert_eq!(text(&session), format!("{HINT}\n"));
    let subagent = run_hook(SYSTEM_PATH, &subagent_json("cook"), repo.path());
    assert_eq!(additional_context(&subagent), HINT);
}

#[test]
fn a_failing_memex_never_fails_the_hook() {
    let repo = fixture();
    let bin = fake_memex("echo boom >&2\nexit 3");
    let session = run_hook(&with_memex(&bin), &session_json(), repo.path());
    assert!(session.status.success(), "the hook exits 0: {session:?}");
    assert_eq!(
        text(&session),
        "",
        "a failed memex prints nothing at SessionStart"
    );
    let subagent = run_hook(&with_memex(&bin), &subagent_json("cook"), repo.path());
    assert_eq!(
        additional_context(&subagent),
        format!("{}\n", id_line("cook"))
    );
}

/// The hook input that Claude Code sends at SessionStart.
fn session_json() -> String {
    r#"{"session_id":"s1","hook_event_name":"SessionStart","source":"startup","cwd":"/fixture"}"#
        .to_string()
}

/// The hook input that Claude Code sends at SubagentStart for a sub-agent of the given type.
fn subagent_json(agent: &str) -> String {
    format!(
        r#"{{"session_id":"s1","hook_event_name":"SubagentStart","agent_id":"a1","agent_type":"{agent}"}}"#
    )
}

/// The id line that a SubagentStart context opens with.
fn id_line(agent: &str) -> String {
    format!("your memex id is {agent}; pass --agent {agent} to memex log and memex info")
}

/// A temporary repo with a `.memex` folder and one README, so that `memex info` reads a small, fixed store.
fn fixture() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary folder");
    fs::create_dir_all(dir.path().join(".memex").join("collections")).expect("the .memex folder");
    fs::write(
        dir.path().join(".memex").join("config.json"),
        "{\"keep_logs\": \"3d\"}\n",
    )
    .expect("config.json");
    fs::write(
        dir.path().join("README.md"),
        "# Fixture\n\nA folder that stands in for a repo in the hook tests.\n",
    )
    .expect("README.md");
    dir
}

/// A folder whose `memex` is a link to the binary under test.
fn memex_link() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary folder");
    symlink(env!("CARGO_BIN_EXE_memex"), dir.path().join("memex"))
        .expect("a link to the memex binary");
    dir
}

/// A folder whose executable `memex` runs `body` as a shell script. The write holds the spawn lock.
fn fake_memex(body: &str) -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let _guard = lock();
    let path = dir.path().join("memex");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("the fake memex is written");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
        .expect("the fake memex is executable");
    dir
}

/// The PATH that finds `memex` in `dir` first, then the system tools.
fn with_memex(dir: &TempDir) -> String {
    format!("{}:{SYSTEM_PATH}", dir.path().display())
}

/// Runs the hook under sh, with `path` as PATH, `stdin` as the hook input and `dir` as the working directory.
fn run_hook(path: &str, stdin: &str, dir: &Path) -> Output {
    let hook = repo().join(HOOK);
    let script = hook.to_str().expect("the hook path is UTF-8");
    run(Path::new("/bin/sh"), &[script], path, dir, stdin)
}

/// Runs the built memex with `args` in `dir`, in the same environment as the hook.
fn run_memex(args: &[&str], dir: &Path) -> Output {
    run(
        Path::new(env!("CARGO_BIN_EXE_memex")),
        args,
        SYSTEM_PATH,
        dir,
        "",
    )
}

/// Runs `program` in `dir` with only PATH and the test clock set, and HOME when there is one, feeding `stdin`.
fn run(program: &Path, args: &[&str], path: &str, dir: &Path, stdin: &str) -> Output {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(dir)
        .env_clear()
        .env("PATH", path)
        .env("MEMEX_NOW", NOW)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(home) = std::env::var_os("HOME") {
        command.env("HOME", home);
    }
    let mut child = {
        let _guard = lock();
        command
            .spawn()
            .unwrap_or_else(|e| panic!("cannot run {}: {e}", program.display()))
    };
    if let Some(mut pipe) = child.stdin.take() {
        // A process that exits without reading its input closes the pipe, which is not a failure here.
        let _ = pipe.write_all(stdin.as_bytes());
    }
    child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("cannot wait for {}: {e}", program.display()))
}

/// The additionalContext of a SubagentStart run. The run must exit 0 and print one JSON line that names the event.
fn additional_context(output: &Output) -> String {
    assert!(output.status.success(), "the hook exits 0: {output:?}");
    let out = text(output);
    assert_eq!(out.lines().count(), 1, "one JSON line: {out:?}");
    let json: Value =
        serde_json::from_str(&out).unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {out:?}"));
    assert_eq!(
        json["hookSpecificOutput"]["hookEventName"], "SubagentStart",
        "{out:?}"
    );
    json["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap_or_else(|| panic!("no additionalContext: {out:?}"))
        .to_string()
}

/// The stdout of a run, as text.
fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The lock that serialises spawns and fake memex writes.
fn lock() -> MutexGuard<'static, ()> {
    SPAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The crate root, where the hook lives.
fn repo() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
