//! Reads a file's header, the leading comment block of a file whose extension uses `//` comments (design §2.1). S2.1 adds the other comment families and the full header rules.
//! Cases: SHOW-FILE-01, SHOW-FILE-02, SHOW-DIR-01, SHOW-LINE-01, SHOW-UP-02

use std::path::Path;

/// The extensions whose header is a `//` comment block (design §2.1).
const SLASH_LANGUAGES: [&str; 14] = [
    "rs", "ts", "tsx", "js", "jsx", "go", "c", "h", "cpp", "hpp", "java", "kt", "swift", "cs",
];

/// The header of the file at `path`, read from its first `bytes`: the text of its leading `//`, `//!` or `///` block, with
/// the prefixes removed. None when the extension is not a `//` language or the file opens with no comment block.
pub fn header(path: &Path, bytes: &[u8]) -> Option<String> {
    let extension = path.extension()?.to_str()?;
    if !SLASH_LANGUAGES.contains(&extension) {
        return None;
    }
    let block = comment_block(&String::from_utf8_lossy(bytes));
    (!block.is_empty()).then_some(block)
}

/// The block's text. Blank lines before the first comment are skipped, the block ends at the first line that is neither a
/// comment nor blank, and trailing blank lines are dropped.
fn comment_block(text: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    let mut started = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(body) = comment_body(trimmed) {
            started = true;
            lines.push(body);
        } else if trimmed.is_empty() {
            if started {
                lines.push("");
            }
        } else {
            break;
        }
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

/// The text of a `//`, `//!` or `///` line after its prefix and one space, or None for any other line.
fn comment_body(line: &str) -> Option<&str> {
    let body = line
        .strip_prefix("//!")
        .or_else(|| line.strip_prefix("///"))
        .or_else(|| line.strip_prefix("//"))?;
    Some(body.strip_prefix(' ').unwrap_or(body).trim_end())
}
