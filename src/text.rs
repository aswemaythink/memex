//! Reads text the way every reader does (design §4.4 Text): a leading UTF-8 byte order mark is dropped.
//! Bytes that are not UTF-8 are decoded lossily.
//! Cases: HDR-BOM-01, HDR-BOM-02, LOG-BODY-06

use std::fs;
use std::io;
use std::path::Path;

/// The UTF-8 byte order mark, as its bytes.
const BOM: &[u8] = b"\xEF\xBB\xBF";

/// The text of `bytes`: a leading byte order mark dropped, and any byte that is not UTF-8 decoded lossily.
pub fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes.strip_prefix(BOM).unwrap_or(bytes)).into_owned()
}

/// The text that is already decoded, without its leading byte order mark.
pub fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// The whole text of the file at `path`, read as `decode` reads bytes.
pub fn read(path: &Path) -> io::Result<String> {
    fs::read(path).map(|bytes| decode(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_leading_byte_order_mark_is_dropped_and_the_rest_is_kept() {
        assert_eq!(decode(b"\xEF\xBB\xBFx\n"), "x\n");
        assert_eq!(decode("\u{feff}\u{feff}x".as_bytes()), "\u{feff}x");
        assert_eq!(strip_bom("\u{feff}# T\n"), "# T\n");
        assert_eq!(strip_bom("# T\n"), "# T\n");
    }

    #[test]
    fn bytes_that_are_not_utf8_are_decoded_lossily() {
        assert_eq!(decode(b"a\xFFb"), "a\u{fffd}b");
    }
}
