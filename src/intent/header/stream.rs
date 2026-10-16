//! Counts a byte stream's lines for `check` (design §3.5d Lines), reading the stream in chunks.

use std::io::{self, Read};

/// The UTF-8 byte order mark. At the start of a stream it is not a line by itself (design §3.5d Lines).
const BOM: &[u8] = b"\xEF\xBB\xBF";

/// The size of the chunks in which a stream is read.
const CHUNK: usize = 8 * 1024;

/// The lines of a file whose first bytes are `head` and whose rest is read from `rest` (design §3.5d Lines). A line ends at LF, so a
/// CRLF counts once and a lone CR ends no line; a last line without LF counts when it is not empty; a leading byte order mark is not
/// a line by itself. `rest` is read in chunks.
pub fn count_lines(head: &[u8], rest: impl Read) -> io::Result<u32> {
    let mut stream = head.chain(rest);
    let mut lead = Vec::with_capacity(BOM.len());
    (&mut stream)
        .take(BOM.len() as u64)
        .read_to_end(&mut lead)?;
    if lead == BOM {
        lead.clear();
    }
    count_stream(lead.as_slice().chain(stream))
}

/// The lines of the bytes that `reader` gives: one for each LF, and one more when the last byte is not LF.
fn count_stream(mut reader: impl Read) -> io::Result<u32> {
    let mut buf = [0u8; CHUNK];
    let mut newlines: u64 = 0;
    let mut last = None;
    loop {
        let read = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(read) => read,
            Err(failure) if failure.kind() == io::ErrorKind::Interrupted => continue,
            Err(failure) => return Err(failure),
        };
        let chunk = &buf[..read];
        newlines += chunk.iter().filter(|&&byte| byte == b'\n').count() as u64;
        last = chunk.last().copied();
    }
    let unended = last.is_some_and(|byte| byte != b'\n');
    Ok(saturate(newlines + u64::from(unended)))
}

/// `count` as a line count. No real file has more lines than a u32 holds, so a larger count stops at the largest one.
fn saturate(count: u64) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reader that is interrupted before each byte it gives, one byte at a time.
    struct Trickle<'a> {
        bytes: &'a [u8],
        calls: usize,
    }

    impl Read for Trickle<'_> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls % 2 == 1 {
                return Err(io::ErrorKind::Interrupted.into());
            }
            match (self.bytes.split_first(), buf.first_mut()) {
                (Some((&byte, rest)), Some(slot)) => {
                    *slot = byte;
                    self.bytes = rest;
                    Ok(1)
                }
                _ => Ok(0),
            }
        }
    }

    /// A reader whose read fails.
    struct Broken;

    impl Read for Broken {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("a failed read"))
        }
    }

    /// The line count of `text` read whole.
    fn lines_of(text: &[u8]) -> u32 {
        count_lines(text, io::empty()).expect("a count")
    }

    #[test]
    fn line_ends_count_as_the_design_says() {
        let cases: [(&[u8], u32); 13] = [
            (b"", 0),
            (b"abc", 1),
            (b"abc\n", 1),
            (b"abc\n\n", 2),
            (b"\n", 1),
            (b"a\r\nb\r\n", 2),
            (b"\r\n", 1),
            (b"a\rb", 1),
            (b"a\rb\n", 1),
            (b"a\nb\r\nc", 3),
            (b"\n\r\n", 2),
            (b"\xEF\xBB\xBF", 0),
            (b"\xEF\xBB\xBFx", 1),
        ];
        for (text, expected) in cases {
            assert_eq!(lines_of(text), expected, "{text:?}");
        }
    }

    #[test]
    fn only_one_leading_byte_order_mark_is_dropped() {
        assert_eq!(lines_of(b"\xEF\xBB\xBF\n"), 1);
        assert_eq!(lines_of(b"\xEF\xBB\xBF\xEF\xBB\xBF"), 1);
        assert_eq!(lines_of(b"\xEF\xBB"), 1, "two bytes are not a mark");
        let split = count_lines(b"\xEF\xBB", &b"\xBF"[..]).expect("a count");
        assert_eq!(split, 0, "a mark split by the head");
    }

    #[test]
    fn a_cut_anywhere_in_the_text_counts_the_same() {
        let text: &[u8] = b"\xEF\xBB\xBFa\r\n\r\nb\rc\n\nlast";
        for cut in 0..=text.len() {
            let counted = count_lines(&text[..cut], &text[cut..]).expect("a count");
            assert_eq!(counted, 5, "cut at {cut}");
        }
    }

    #[test]
    fn short_or_interrupted_reads_count_the_same() {
        let whole = Trickle {
            bytes: b"a\r\nb\n\nc",
            calls: 0,
        };
        assert_eq!(count_lines(b"", whole).expect("a count"), 4);
        let rest = Trickle {
            bytes: b"\nb\n\nc",
            calls: 0,
        };
        assert_eq!(count_lines(b"a\r", rest).expect("a count"), 4);
    }

    #[test]
    fn a_failed_read_is_an_error() {
        assert!(count_lines(b"abc", Broken).is_err());
    }
}
