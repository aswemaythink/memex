//! Reads a file's header (design §2.1): its leading comment block, as the text that `show` prints.
//! Cases: HDR-BOM-01, HDR-BIN-01, HDR-EXT-01, HDR-CAP-01, HDR-CAP-02, HDR-NONE-01, HDR-CRLF-01, HDR-LIMIT-01

mod families;
mod scan;

use std::path::Path;

/// The UTF-8 byte order mark, which is dropped on read (design §4.4).
const BOM: &[u8] = b"\xEF\xBB\xBF";

/// The header of the file at `path`, read from its first bytes (at most 8 KiB, see `corpus.rs`): the text of its leading comment
/// block with the markers removed. None for a file with no comment family, a binary file (one with a NUL byte), and a file whose
/// first block is absent.
pub fn header(path: &Path, bytes: &[u8]) -> Option<String> {
    let family = families::family_of(path)?;
    let bytes = bytes.strip_prefix(BOM).unwrap_or(bytes);
    if bytes.contains(&0) {
        return None;
    }
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().collect();
    scan::header_text(family, &lines)
}

#[cfg(test)]
mod tests {
    use super::*;

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
