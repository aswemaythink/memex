//! Reads the log body from stdin and checks it (design §3.4 Body and §4.4 Text): a terminal is refused, the read stops one byte past 16 KiB, a leading UTF-8 BOM is dropped, a blank body is refused, and the body needs a title line.
//! Cases: LOG-WRITE-12, LOG-BODY-01, LOG-BODY-02, LOG-BODY-03, LOG-BODY-04, LOG-BODY-05, LOG-BODY-06; the terminal refusal has a unit test only (design §4.4)

use std::io::{self, IsTerminal, Read};

use crate::cli::Verb;
use crate::error::Error;
use crate::logs::write::title_of;
use crate::scan::fail;

/// The most bytes a body may have, counted on the raw stdin bytes (design §3.4): 16 KiB.
pub const LIMIT: usize = 16 * 1024;

/// The usage message for a body typed at a terminal, where there is no heredoc to end.
const TERMINAL: &str = "log body comes from stdin: memex log <stream> <<'EOF' … EOF";

/// The usage message for a body over `LIMIT`: a log holds findings, so artifacts are cited by path.
const OVER: &str = "log body is over 16 KiB: logs are findings: cite artifacts by path";

/// The UTF-8 byte order mark, which a shell may put at the start of a body. It is dropped on read (design §4.4 Text).
const BOM: char = '\u{feff}';

/// Reads the body that stdin carries, under the rules of `read`.
pub fn read_stdin() -> Result<String, Error> {
    let stdin = io::stdin();
    read(stdin.is_terminal(), stdin.lock())
}

/// The body that `input` carries, with CRLF turned into LF and a leading BOM dropped. `terminal` says whether `input` is a
/// terminal, which is refused before anything is read. A body over `LIMIT` bytes, or one that is blank, is a usage error.
pub fn read(terminal: bool, input: impl Read) -> Result<String, Error> {
    if terminal {
        return Err(fail(Verb::Log, TERMINAL.to_string()));
    }
    let mut bytes = Vec::new();
    // One byte past the limit is read, so a longer body shows as longer without reading all of it.
    input
        .take(LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|failure| Error::io("cannot read the log body", &failure))?;
    if bytes.len() > LIMIT {
        return Err(fail(Verb::Log, OVER.to_string()));
    }
    let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
    let body = text.strip_prefix(BOM).unwrap_or(&text);
    if body.trim().is_empty() {
        return Err(fail(Verb::Log, "log body is empty".to_string()));
    }
    Ok(body.to_string())
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
    fn blank_bodies_are_empty_and_crlf_becomes_lf() {
        let blank = read(false, &b" \r\n\t\n"[..]).unwrap_err();
        assert_eq!(blank.message, "log body is empty");
        assert_eq!(read(false, &b"a\r\nb\r\n"[..]).unwrap(), "a\nb\n");
    }

    #[test]
    fn a_leading_bom_is_dropped_once() {
        assert_eq!(read(false, "\u{feff}# T\n".as_bytes()).unwrap(), "# T\n");
        let second = read(false, "\u{feff}\u{feff}x\n".as_bytes()).unwrap();
        assert_eq!(second, "\u{feff}x\n");
        let blank = read(false, "\u{feff} \n".as_bytes()).unwrap_err();
        assert_eq!(blank.message, "log body is empty");
    }

    #[test]
    fn a_title_needs_text() {
        assert!(check_title("## Heading\n").is_ok());
        let marks = check_title("#  #\n").unwrap_err();
        assert_eq!(marks.message, "log body has no title line");
        assert!(check_title(" \n\t\n").is_err());
    }
}
