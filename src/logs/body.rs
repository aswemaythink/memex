//! Reads the log body from stdin and checks it (design §3.4 Body and §4.4 Text).
//! A terminal is refused, a leading UTF-8 BOM is dropped and CRLF read as LF, the 16 KiB limit is measured on that stored body, a blank body is refused, and the body needs a title line.
//! Cases: LOG-WRITE-09, LOG-WRITE-12, LOG-BODY-01, LOG-BODY-02, LOG-BODY-03, LOG-BODY-04, LOG-BODY-05, LOG-BODY-06, LOG-BODY-07, LOG-BODY-08, PORT-CRLF-06, CLI-HINT-05, CLI-HINT-06; the terminal refusal has a unit test only (design §4.4)

use std::io::{self, IsTerminal, Read};

use crate::cli::Verb;
use crate::error::Error;
use crate::logs::write::title_of;
use crate::scan::fail;
use crate::text;

/// The most bytes a body may store, counted after the BOM is dropped, CRLF is read as LF and the bytes are decoded (design §3.4): 16 KiB.
const LIMIT: usize = 16 * 1024;

/// The most raw bytes a body may read. The BOM is three bytes, CRLF to LF at most halves the rest, and decoding never shrinks
/// it, so a longer read stores as more than `LIMIT` bytes whatever it holds.
const RAW_LIMIT: usize = 2 * LIMIT + 3;

/// The usage message for a body typed at a terminal, where there is no heredoc to end.
const TERMINAL: &str = "log body comes from stdin: memex log <stream> <<'EOF' … EOF";

/// The usage message for a body over `LIMIT`: a log holds findings, so artifacts are cited by path.
const OVER: &str = "log body is over 16 KiB: logs are findings: cite artifacts by path";

/// The usage message for a blank body: it names the command that takes a body, with its heredoc (design §3.5c).
const EMPTY: &str = "log body is empty: memex log <stream> <<'EOF' … EOF";

/// The usage error for a body over `LIMIT` once stored.
fn over() -> Error {
    fail(Verb::Log, OVER.to_string())
}

/// Reads the body that stdin carries, under the rules of `read`.
pub fn read_stdin() -> Result<String, Error> {
    let stdin = io::stdin();
    read(stdin.is_terminal(), stdin.lock())
}

/// The body that `input` carries, with CRLF turned into LF and a leading BOM dropped. `terminal` says whether `input` is a
/// terminal, which is refused before anything is read. A body whose stored form is over `LIMIT` bytes, or one that is blank,
/// is a usage error.
fn read(terminal: bool, input: impl Read) -> Result<String, Error> {
    if terminal {
        return Err(fail(Verb::Log, TERMINAL.to_string()));
    }
    let mut bytes = Vec::new();
    // One byte past the raw limit is read, so a longer body shows as longer without reading all of it.
    input
        .take(RAW_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|failure| Error::io("cannot read the log body", &failure))?;
    if bytes.len() > RAW_LIMIT {
        return Err(over());
    }
    let body = text::decode(&bytes);
    if body.len() > LIMIT {
        return Err(over());
    }
    if body.trim().is_empty() {
        return Err(fail(Verb::Log, EMPTY.to_string()));
    }
    Ok(body)
}

/// Refuses a body with no title: the first non-blank line after the agent block, with its `#`s and spaces removed,
/// must have text (design §3.4). `rest` is the body after that block, as `front::split` leaves it.
pub fn check_title(rest: &str) -> Result<(), Error> {
    if title_of(rest).is_empty() {
        return Err(fail(Verb::Log, "log body has no title line".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Code;

    #[test]
    fn a_terminal_is_refused_before_anything_is_read() {
        let error = read(true, &b"never read"[..]).unwrap_err();
        assert_eq!(error.code, Code::Usage);
        assert_eq!(error.message, TERMINAL);
    }

    #[test]
    fn the_limit_is_inclusive() {
        let exact = vec![b'a'; LIMIT];
        assert!(read(false, exact.as_slice()).is_ok());
        let over = vec![b'a'; LIMIT + 1];
        assert_eq!(read(false, over.as_slice()).unwrap_err().message, OVER);
    }

    #[test]
    fn the_limit_is_measured_on_the_stored_body() {
        // Each "a\r\n" stores as "a\n": 8192 of them are 16 KiB stored and 24 KiB raw, so the raw size is not the test.
        let stored_exact = b"a\r\n".repeat(LIMIT / 2);
        assert!(read(false, stored_exact.as_slice()).is_ok());
        let stored_over = b"a\r\n".repeat(LIMIT / 2 + 1);
        assert_eq!(
            read(false, stored_over.as_slice()).unwrap_err().message,
            OVER
        );
    }

    #[test]
    fn a_body_that_decodes_over_the_limit_is_refused() {
        // Two bytes of a split character become one U+FFFD, three bytes, so the stored body is one byte over.
        let mut bytes = vec![b'a'; LIMIT - 2];
        bytes.extend_from_slice(b"\xe2\x82");
        assert_eq!(read(false, bytes.as_slice()).unwrap_err().message, OVER);
    }

    #[test]
    fn a_read_past_the_raw_limit_is_refused_whatever_it_holds() {
        let raw = vec![b'a'; RAW_LIMIT + 1];
        assert_eq!(read(false, raw.as_slice()).unwrap_err().message, OVER);
    }

    #[test]
    fn blank_bodies_are_empty_and_crlf_becomes_lf() {
        let blank = read(false, &b" \r\n\t\n"[..]).unwrap_err();
        assert_eq!(blank.message, EMPTY);
        assert_eq!(read(false, &b"a\r\nb\r\n"[..]).unwrap(), "a\nb\n");
    }

    #[test]
    fn a_leading_bom_is_dropped_once() {
        assert_eq!(read(false, "\u{feff}# T\n".as_bytes()).unwrap(), "# T\n");
        let second = read(false, "\u{feff}\u{feff}x\n".as_bytes()).unwrap();
        assert_eq!(second, "\u{feff}x\n");
        let blank = read(false, "\u{feff} \n".as_bytes()).unwrap_err();
        assert_eq!(blank.message, EMPTY);
    }

    #[test]
    fn a_title_needs_text() {
        assert!(check_title("## Heading\n").is_ok());
        let marks = check_title("#  #\n").unwrap_err();
        assert_eq!(marks.message, "log body has no title line");
        assert!(check_title(" \n\t\n").is_err());
    }
}
