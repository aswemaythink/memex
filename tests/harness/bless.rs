//! Compares a case's rendered transcript with its expect.txt.
//! Under MEMEX_BLESS=1 it writes the golden instead of failing.
//! Cases: all (runner)

use std::fs;
use std::io;
use std::path::Path;

use super::conform::placeholder_problems;

/// Passes when the golden matches the rendered text. Otherwise fails with a diff and the bless line.
///
/// With MEMEX_BLESS=1 it writes the golden and passes instead. Blessing is refused when CI is set,
/// and refused when the golden would break rule 5.
pub fn judge(case_id: &str, expect: &Path, rendered: &str, shown: &str) -> Result<(), String> {
    let existing = match fs::read(expect) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("cannot read expect.txt ({:?})", e.kind())),
    };
    if existing.as_deref() == Some(rendered.as_bytes()) {
        return Ok(());
    }
    let bless_line = format!("bless: MEMEX_BLESS=1 cargo test --test conformance -- {case_id}");
    let problem = match &existing {
        None => format!(
            "missing expect.txt: run MEMEX_BLESS=1 cargo test --test conformance -- {case_id}"
        ),
        Some(bytes) => format!(
            "{}\n{bless_line}",
            unified_diff(bytes, rendered).trim_end_matches('\n')
        ),
    };
    if !std::env::var("MEMEX_BLESS").is_ok_and(|value| value == "1") {
        return Err(problem);
    }
    if std::env::var_os("CI").is_some() {
        return Err(format!("bless refused: CI is set\n{problem}"));
    }
    if let Some((token, line)) = placeholder_problems(rendered.as_bytes()).into_iter().next() {
        return Err(format!(
            "bless refused: placeholder {token:?} on line {line} is not allowed in a golden; the case must not print it"
        ));
    }
    fs::write(expect, rendered.as_bytes())
        .map_err(|e| format!("cannot write expect.txt ({:?})", e.kind()))?;
    println!("blessed {shown}");
    Ok(())
}

/// A unified diff from the golden on disk to the rendered text.
fn unified_diff(expected: &[u8], actual: &str) -> String {
    let expected = String::from_utf8_lossy(expected);
    similar::TextDiff::from_lines(expected.as_ref(), actual)
        .unified_diff()
        .header("expect.txt", "actual")
        .to_string()
}
