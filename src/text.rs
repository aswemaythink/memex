//! Reads text for every reader (design §4.4): a BOM is dropped and CRLF line ends read as LF.
//! Bytes that are not UTF-8 are decoded lossily. `line_of` gives a line of split text without its line end, for the frontmatter splitters in `logs` and `collections`.
//! Cases: HDR-BOM-01, HDR-BOM-02, HDR-CRLF-01, LOG-BODY-06, LOG-FRONT-01, LOG-FRONT-12, PORT-CRLF-01, PORT-CRLF-02, PORT-CRLF-03, PORT-CRLF-04, PORT-CRLF-05, PORT-CRLF-06, PORT-BOM-01, PORT-BOM-02, PORT-DECODE-01, PORT-DECODE-02

use std::fs;
use std::io;
use std::path::Path;

/// The UTF-8 byte order mark, as its bytes.
const BOM: &[u8] = b"\xEF\xBB\xBF";

/// The text of `bytes`: a leading byte order mark dropped, CRLF line ends made LF, and any byte that is not UTF-8 decoded
/// lossily. A lone carriage return is kept.
pub fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes.strip_prefix(BOM).unwrap_or(bytes)).replace("\r\n", "\n")
}

/// The whole text of the file at `path`, read as `decode` reads bytes.
pub fn read(path: &Path) -> io::Result<String> {
    fs::read(path).map(|bytes| decode(&bytes))
}

/// A piece's text without its line end: one `\n` and a `\r` before it. A `\r` anywhere else is kept.
pub fn line_of(piece: &str) -> &str {
    let line = piece.strip_suffix('\n').unwrap_or(piece);
    line.strip_suffix('\r').unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_loses_its_line_end_and_a_carriage_return_inside_it_is_kept() {
        assert_eq!(line_of("a\r\n"), "a");
        assert_eq!(line_of("a\n"), "a");
        assert_eq!(line_of("a"), "a");
        assert_eq!(line_of("a\rb\n"), "a\rb");
    }

    #[test]
    fn one_leading_byte_order_mark_is_dropped_and_the_rest_is_kept() {
        assert_eq!(decode(b"\xEF\xBB\xBFx\n"), "x\n");
        assert_eq!(decode("\u{feff}\u{feff}x".as_bytes()), "\u{feff}x");
    }

    #[test]
    fn bytes_that_are_not_utf8_are_decoded_lossily() {
        assert_eq!(decode(b"a\xFFb"), "a\u{fffd}b");
    }

    #[test]
    fn crlf_line_ends_are_read_as_lf_and_a_lone_carriage_return_is_kept() {
        assert_eq!(decode(b"a\r\n\r\nb\r\n"), "a\n\nb\n");
        assert_eq!(decode(b"a\rb\n"), "a\rb\n");
    }
}
