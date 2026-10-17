//! Renders a case run as the golden transcript.
//! It redacts the sandbox path and the crate version.
//! Cases: all (runner)

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// What one step did, as the transcript shows it.
pub struct StepRecord {
    pub args: Vec<String>,
    /// The cwd as case.json wrote it; "repo" when the step names none.
    pub cwd: String,
    /// The step's own env, not the case's.
    pub env: BTreeMap<String, String>,
    /// None when a signal ended the process.
    pub exit: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// The marker line for text that does not end in a newline.
const NO_NEWLINE: &str = "\\ no newline at end";

/// Renders every step (one blank line apart), then each snapshot path in order.
pub fn render(
    steps: &[StepRecord],
    sandbox: &Path,
    snapshots: &[String],
) -> Result<String, String> {
    let mut lines: Vec<String> = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(command_line(step));
        lines.push(match step.exit {
            Some(code) => format!("exit {code}"),
            None => "exit signal".to_string(),
        });
        lines.extend(stream("stdout", &step.stdout));
        lines.extend(stream("stderr", &step.stderr));
    }
    for snapshot in snapshots {
        lines.extend(render_snapshot(sandbox, snapshot)?);
    }
    let mut text = lines.join("\n");
    text.push('\n');
    Ok(text)
}

/// `$ memex <args>`, plus `  [cwd X]` when the cwd is not repo, and `  [env K=V ...]` for the step's own env.
fn command_line(step: &StepRecord) -> String {
    let mut line = String::from("$ memex");
    for arg in &step.args {
        line.push(' ');
        line.push_str(&quote(arg));
    }
    if step.cwd != "repo" {
        line.push_str(&format!("  [cwd {}]", quote(&step.cwd)));
    }
    if !step.env.is_empty() {
        let pairs: Vec<String> = step
            .env
            .iter()
            .map(|(key, value)| quote(&format!("{key}={value}")))
            .collect();
        line.push_str(&format!("  [env {}]", pairs.join(" ")));
    }
    line
}

/// Single-quotes an argument unless it is made only of `[A-Za-z0-9._/:@%+=,-]`.
fn quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._/:@%+=,-".contains(&b));
    if plain {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

/// A stream's section: `stdout (empty)`, `stdout (binary)`, or the label then its `| ` rows.
fn stream(label: &str, bytes: &[u8]) -> Vec<String> {
    if bytes.is_empty() {
        return vec![format!("{label} (empty)")];
    }
    match text_of(bytes) {
        Some(text) => {
            let mut lines = vec![label.to_string()];
            lines.extend(rows(text));
            lines
        }
        None => vec![format!("{label} (binary)")],
    }
}

/// The bytes as text, or None when they hold a NUL byte or are not UTF-8.
fn text_of(bytes: &[u8]) -> Option<&str> {
    if bytes.contains(&0) {
        return None;
    }
    std::str::from_utf8(bytes).ok()
}

/// Every line as a `| ` row, plus the no-newline marker when the text does not end in `\n`.
fn rows(text: &str) -> Vec<String> {
    let (body, ends_with_newline) = match text.strip_suffix('\n') {
        Some(body) => (body, true),
        None => (text, false),
    };
    let mut lines: Vec<String> = body.split('\n').map(|line| format!("| {line}")).collect();
    if !ends_with_newline {
        lines.push(NO_NEWLINE.to_string());
    }
    lines
}

/// `== snapshot <path>`, then each file under it as `-- <path>` and its rows, or `(missing)` or `(empty)`.
fn render_snapshot(sandbox: &Path, snapshot: &str) -> Result<Vec<String>, String> {
    let mut lines = vec![format!("== snapshot {snapshot}")];
    let root = sandbox.join(snapshot);
    let meta = match fs::metadata(&root) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            lines.push("(missing)".to_string());
            return Ok(lines);
        }
        Err(e) => return Err(io_failure(&format!("cannot read snapshot {snapshot}"), &e)),
    };
    if meta.is_file() {
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        lines.push(format!("-- {name}"));
        lines.extend(file_rows(&root)?);
        return Ok(lines);
    }
    let mut files = Vec::new();
    collect_files(&root, "", &mut files)?;
    if files.is_empty() {
        lines.push("(empty)".to_string());
        return Ok(lines);
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    for (relative, file) in files {
        lines.push(format!("-- {relative}"));
        match file {
            Entry::File(path) => lines.extend(file_rows(&path)?),
            Entry::Other(label) => lines.push(label.to_string()),
        }
    }
    Ok(lines)
}

/// A file found in a snapshot, or something else that is listed but not read.
enum Entry {
    File(PathBuf),
    Other(&'static str),
}

/// Walks a folder without following symlinks, collecting (`/`-separated relative path, entry).
fn collect_files(dir: &Path, prefix: &str, out: &mut Vec<(String, Entry)>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| io_failure("cannot read a snapshot folder", &e))?;
    for entry in entries {
        let entry = entry.map_err(|e| io_failure("cannot read a snapshot folder", &e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let relative = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        let kind = entry
            .file_type()
            .map_err(|e| io_failure("cannot read a snapshot folder", &e))?;
        if kind.is_dir() {
            collect_files(&entry.path(), &relative, out)?;
        } else if kind.is_file() {
            out.push((relative, Entry::File(entry.path())));
        } else if kind.is_symlink() {
            out.push((relative, Entry::Other("(symlink)")));
        } else {
            out.push((relative, Entry::Other("(not a file)")));
        }
    }
    Ok(())
}

/// The rows of one snapshot file: `(empty)` for no bytes, `(binary)` for a NUL or non-UTF-8 file.
fn file_rows(path: &Path) -> Result<Vec<String>, String> {
    let bytes = fs::read(path).map_err(|e| io_failure("cannot read a snapshot file", &e))?;
    if bytes.is_empty() {
        return Ok(vec!["(empty)".to_string()]);
    }
    Ok(match text_of(&bytes) {
        Some(text) => rows(text),
        None => vec!["(binary)".to_string()],
    })
}

/// Every spelling of the sandbox path that output may show: raw, canonical and dunce-simplified,
/// each with `\` and with `/`. Longest first, so a longer spelling is never cut short.
pub fn sandbox_spellings(sandbox: &Path) -> Vec<String> {
    let mut forms = vec![sandbox.to_string_lossy().into_owned()];
    if let Ok(path) = fs::canonicalize(sandbox) {
        forms.push(path.to_string_lossy().into_owned());
    }
    if let Ok(path) = dunce::canonicalize(sandbox) {
        forms.push(path.to_string_lossy().into_owned());
    }
    let mut spellings = Vec::new();
    for form in forms {
        spellings.push(form.replace('\\', "/"));
        spellings.push(form.replace('/', "\\"));
        spellings.push(form);
    }
    spellings.retain(|s| !s.is_empty());
    spellings.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    spellings.dedup();
    spellings
}

/// Replaces each sandbox spelling with `$SANDBOX`, then the crate version with `$VERSION`.
pub fn redact(text: &str, spellings: &[String], version: &str) -> String {
    let mut out = text.to_string();
    for spelling in spellings {
        out = out.replace(spelling.as_str(), "$SANDBOX");
    }
    redact_version(&out, version)
}

/// Replaces the version only where it stands alone as a version, so `0.1.0` is redacted
/// but `10.1.0` and `0.1.0.5` are left as they are.
fn redact_version(text: &str, version: &str) -> String {
    if version.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(version) {
        let (before, after) = (&rest[..at], &rest[at + version.len()..]);
        let joined_before = before.ends_with(|c: char| c.is_ascii_digit() || c == '.');
        let mut next = after.chars();
        let joined_after = match next.next() {
            Some(c) if c.is_ascii_digit() => true,
            Some('.') => next.next().is_some_and(|c| c.is_ascii_digit()),
            _ => false,
        };
        out.push_str(before);
        out.push_str(if joined_before || joined_after {
            version
        } else {
            "$VERSION"
        });
        rest = after;
    }
    out.push_str(rest);
    out
}

fn io_failure(what: &str, error: &io::Error) -> String {
    format!("{what} ({:?})", error.kind())
}

/// Unit tests for the rendering rules. They run in runner_selftest; conformance.rs (harness = false) never compiles them in.
#[cfg(test)]
mod tests {
    #[test]
    fn plain_args_stay_bare_and_others_are_quoted() {
        use super::*;
        assert_eq!(quote("--tag"), "--tag");
        assert_eq!(quote("a b"), "'a b'");
        assert_eq!(quote(""), "''");
        assert_eq!(quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn rows_mark_a_missing_final_newline() {
        use super::*;
        assert_eq!(rows("a\n\nb"), ["| a", "| ", "| b", "\\ no newline at end"]);
        assert_eq!(rows("\n"), ["| "]);
    }

    #[test]
    fn streams_render_empty_and_binary_as_one_line() {
        use super::*;
        assert_eq!(stream("stderr", b""), ["stderr (empty)"]);
        assert_eq!(stream("stdout", b"\x00x"), ["stdout (binary)"]);
        assert_eq!(stream("stdout", &[0xff]), ["stdout (binary)"]);
        assert_eq!(stream("stdout", b"hi\n"), ["stdout", "| hi"]);
    }

    #[test]
    fn redaction_takes_the_longest_spelling_first() {
        use super::*;
        let spellings = vec!["/private/var/x".to_string(), "/var/x".to_string()];
        assert_eq!(
            redact("/private/var/x/repo and 0.1.0", &spellings, "0.1.0"),
            "$SANDBOX/repo and $VERSION"
        );
    }

    #[test]
    fn version_redaction_leaves_longer_numbers_alone() {
        use super::*;
        assert_eq!(
            redact_version("v0.1.0, 10.1.0, 0.1.0.5, 0.1.0.\n", "0.1.0"),
            "v$VERSION, 10.1.0, 0.1.0.5, $VERSION.\n"
        );
    }

    #[test]
    fn command_lines_quote_cwd_and_env() {
        use super::*;
        let step = StepRecord {
            args: vec!["show".to_string()],
            cwd: "a b".to_string(),
            env: BTreeMap::from([("K".to_string(), "x y".to_string())]),
            exit: Some(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        assert_eq!(
            command_line(&step),
            "$ memex show  [cwd 'a b']  [env 'K=x y']"
        );
    }
}
