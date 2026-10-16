//! Reads a file's header (design §2.1): its leading comment block, as the text that `show` prints.
//! Cases: HDR-BOM-01, HDR-BOM-02, HDR-BIN-01, HDR-EXT-01, HDR-CAP-01, HDR-CAP-02, HDR-CAP-03, HDR-CAP-04, HDR-CAP-05, HDR-NONE-01, HDR-CRLF-01, HDR-LIMIT-01

mod families;
#[cfg(test)]
mod lines;
mod scan;
#[cfg(test)]
mod stream;

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use crate::text;

/// The most bytes of a file that header detection reads, which a merge reads of a code file too (design §2.1).
const HEAD_BYTES: u64 = 8 * 1024;

/// The last line of a header that is cut at the read limit. It follows a blank line, so it is never part of a one-liner.
const CUT_NOTE: &str = "… (header cut at 8 KiB)";

/// The first `HEAD_BYTES` of the file at `path`: the bytes that header detection reads, and no more (design §2.1).
pub fn read_head(path: &Path) -> io::Result<Vec<u8>> {
    let mut head = Vec::new();
    File::open(path)?.take(HEAD_BYTES).read_to_end(&mut head)?;
    Ok(head)
}

/// The header of the file at `path`, read from its first bytes (at most 8 KiB, see `corpus.rs`): the text of its leading comment
/// block with the markers removed. A header still open at the read limit ends with the cut note (design §2.1). None for a file with
/// no comment family, a binary file (one with a NUL byte), and a file whose first block is absent.
pub fn header(path: &Path, bytes: &[u8]) -> Option<String> {
    let family = families::family_of(path)?;
    if bytes.contains(&0) {
        return None;
    }
    let text = text::decode(bytes);
    let lines: Vec<&str> = text.lines().collect();
    let found = scan::header_text(family, &lines)?;
    Some(if found.open && is_longer(path, bytes) {
        format!("{}\n\n{CUT_NOTE}", found.text)
    } else {
        found.text
    })
}

/// True when a file of this name or extension has a comment family, so that its header is read. Any other file has no header.
pub fn has_family(path: &Path) -> bool {
    families::family_of(path).is_some()
}

/// The header text as the index reads it: with the cut note left out, since only `show` prints that line (design §2.1).
pub fn without_note(text: &str) -> &str {
    text.strip_suffix(CUT_NOTE)
        .and_then(|rest| rest.strip_suffix("\n\n"))
        .unwrap_or(text)
}

/// True when the file is longer than the bytes read from it, so a header open at their end goes on past the read limit (design
/// §2.1). The length comes from the file's metadata, since a file of exactly the read limit has nothing more to read.
fn is_longer(path: &Path, bytes: &[u8]) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.len() > bytes.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_header_still_open_at_the_read_limit_ends_with_the_cut_note() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("long.rs");
        let body = format!("//! {}", "word ".repeat(2_000));
        fs::write(&path, &body).expect("a written file");
        let head = &body.as_bytes()[..8 * 1024];
        let text = header(&path, head).expect("a header");
        assert!(text.ends_with(&format!("\n\n{CUT_NOTE}")));
        assert!(!without_note(&text).contains(CUT_NOTE));
    }

    #[test]
    fn a_header_that_ends_inside_the_read_has_no_note() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("short.rs");
        let body = format!("//! Short.\n{}", "fn x() {}\n".repeat(1_000));
        fs::write(&path, &body).expect("a written file");
        let head = &body.as_bytes()[..8 * 1024];
        assert_eq!(header(&path, head), Some("Short.".to_string()));
    }

    #[test]
    fn a_file_of_exactly_the_read_limit_has_no_note() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("exact.rs");
        let body = format!("//! {}", "w".repeat(8 * 1024 - 4));
        fs::write(&path, &body).expect("a written file");
        let text = header(&path, body.as_bytes()).expect("a header");
        assert!(!text.contains(CUT_NOTE));
    }

    #[test]
    fn a_nul_byte_makes_the_file_binary() {
        assert_eq!(header(Path::new("blob.rs"), b"//! Text.\n\0"), None);
        assert_eq!(
            header(Path::new("text.rs"), b"//! Text.\n"),
            Some("Text.".to_string())
        );
    }

    #[test]
    fn a_byte_order_mark_is_dropped() {
        let bytes = b"\xEF\xBB\xBF//! Purpose.\n";
        assert_eq!(
            header(Path::new("bom.rs"), bytes),
            Some("Purpose.".to_string())
        );
    }

    #[test]
    fn a_file_name_is_matched_exactly_before_the_extension() {
        assert_eq!(
            header(Path::new("Makefile"), b"# Build.\n"),
            Some("Build.".to_string())
        );
        assert_eq!(header(Path::new("makefile.txt"), b"# Build.\n"), None);
    }
}
