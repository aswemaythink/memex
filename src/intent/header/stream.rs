//! Counts a file's lines for `check` (design §3.5d Lines).
//! The head is tallied as the corpus reads it; a longer file's rest carries the tally on, read in chunks.

use std::io::{self, Read};

/// The UTF-8 byte order mark. At the start of a stream it is not a line by itself (design §3.5d Lines).
const BOM: &[u8] = b"\xEF\xBB\xBF";

/// The size of the chunks in which a stream is read.
const CHUNK: usize = 8 * 1024;

/// The line count of a file so far: its LFs, and its last byte, which says whether a last line is still open (design §3.5d Lines).
/// A line ends at LF, so a CRLF counts once and a lone CR ends no line; a last line without LF counts when it is not empty.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    /// The LFs read so far.
    newlines: u64,
    /// The last byte read so far, None before any.
    last: Option<u8>,
}

impl Tally {
    /// The tally of a file's head, its first bytes. A leading byte order mark is not a line by itself. A head shorter than the mark
    /// is the whole file, since the head is cut at 8 KiB, so the mark is never split between the head and the rest.
    pub fn of_head(head: &[u8]) -> Tally {
        let text = head.strip_prefix(BOM).unwrap_or(head);
        Tally {
            newlines: newlines_in(text),
            last: text.last().copied(),
        }
    }

    /// This tally carried on through the bytes that `rest` gives, read in chunks.
    pub fn read_on(mut self, mut rest: impl Read) -> io::Result<Tally> {
        let mut buf = [0u8; CHUNK];
        loop {
            let read = match rest.read(&mut buf) {
                Ok(0) => return Ok(self),
                Ok(read) => read,
                Err(failure) if failure.kind() == io::ErrorKind::Interrupted => continue,
                Err(failure) => return Err(failure),
            };
            let chunk = &buf[..read];
            self.newlines += newlines_in(chunk);
            self.last = chunk.last().copied();
        }
    }

    /// The lines counted: one for each LF, and one more when the last byte is not LF. No real file has more lines than a u32
    /// holds, so a larger count stops at the largest one.
    pub fn lines(self) -> u32 {
        let unended = self.last.is_some_and(|byte| byte != b'\n');
        u32::try_from(self.newlines + u64::from(unended)).unwrap_or(u32::MAX)
    }
}

/// The LFs in `bytes`.
fn newlines_in(bytes: &[u8]) -> u64 {
    bytes.iter().filter(|&&byte| byte == b'\n').count() as u64
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

    /// The line count of `text` read whole, as a head.
    fn lines_of(text: &[u8]) -> u32 {
        Tally::of_head(text).lines()
    }

    /// The line count of `head` carried on through `rest`.
    fn count_lines(head: &[u8], rest: impl Read) -> io::Result<u32> {
        Tally::of_head(head).read_on(rest).map(Tally::lines)
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
    }

    #[test]
    fn a_cut_anywhere_after_the_mark_counts_the_same() {
        let text: &[u8] = b"\xEF\xBB\xBFa\r\n\r\nb\rc\n\nlast";
        for cut in BOM.len()..=text.len() {
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
