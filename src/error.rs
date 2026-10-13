//! The fatal error of design §3.6: its five codes, their exit codes, and the text and JSON renderings.
//! Cases: CLI-EXIT-*, CLI-JSON-*, DISC-ROOT-04, LOG-WRITE-11

use std::io::{self, Write};

/// The code of a fatal error, as design §3.6 names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    /// The `usage` code: exit 2.
    Usage,
    /// The `not_found` code: exit 1.
    NotFound,
    /// The `ambiguous` code: exit 1.
    Ambiguous,
    /// The `no_root` code: exit 3.
    NoRoot,
    /// The `io` code: exit 3.
    Io,
}

impl Code {
    /// The code as the JSON envelope spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            Code::Usage => "usage",
            Code::NotFound => "not_found",
            Code::Ambiguous => "ambiguous",
            Code::NoRoot => "no_root",
            Code::Io => "io",
        }
    }

    /// The exit status: usage is 2; not_found and ambiguous are 1; no_root and io are 3.
    pub fn exit_code(&self) -> i32 {
        match self {
            Code::Usage => 2,
            Code::NotFound | Code::Ambiguous => 1,
            Code::NoRoot | Code::Io => 3,
        }
    }
}

/// A fatal error: a code, a message, and the lines printed after the message on stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// Sets the JSON `code` and the exit status.
    pub code: Code,
    /// Printed as `memex: <message>`. Empty when the error is only its notes.
    pub message: String,
    /// Printed after the message in text mode, one line each, such as `usage: <grammar line>`.
    pub notes: Vec<String>,
}

impl Error {
    /// A usage error: exit 2, with the lines that follow its message.
    pub fn usage(message: String, notes: Vec<String>) -> Error {
        Error {
            code: Code::Usage,
            message,
            notes,
        }
    }

    /// An I/O error: exit 3. It names what failed and the io::ErrorKind, never the OS text (PLAN §1 fix 17).
    pub fn io(what: &str, error: &io::Error) -> Error {
        Error {
            code: Code::Io,
            message: format!("{what} ({:?})", error.kind()),
            notes: Vec::new(),
        }
    }

    /// An environment error with a message memex wrote itself, such as an invalid MEMEX_NOW: exit 3, code `io`.
    pub fn io_message(message: String) -> Error {
        Error {
            code: Code::Io,
            message,
            notes: Vec::new(),
        }
    }

    /// The `no_root` error of design §4.1: `log` finds no `.memex/` in the root or the main checkout. Exit 3.
    pub fn no_root() -> Error {
        Error {
            code: Code::NoRoot,
            message: "no .memex/ here: run `memex init`".to_string(),
            notes: Vec::new(),
        }
    }

    /// The exit status of this error.
    pub fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }

    /// The text form: the message on a `memex:` line, then each note on its own line.
    pub fn text(&self) -> String {
        let mut lines = Vec::new();
        if !self.message.is_empty() {
            lines.push(format!("memex: {}", self.message));
        }
        lines.extend(self.notes.iter().cloned());
        let mut text = lines.join("\n");
        text.push('\n');
        text
    }

    /// The JSON envelope of design §3.6, `{"error":{"code":...,"message":...}}`, without a newline.
    pub fn json(&self) -> String {
        format!(
            "{{\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}}}}",
            self.code.as_str(),
            escape(&self.message)
        )
    }
}

/// Escapes text for a JSON string: quotes, backslashes and control characters.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            ch if ch < ' ' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

/// Prints an error the way the invocation asked for, and returns its exit status.
///
/// With `json`, the envelope goes to stdout and stderr stays empty; otherwise the text goes to stderr.
/// A failed write is ignored, since a closed stream leaves nowhere to report it.
pub fn report(error: &Error, json: bool) -> i32 {
    if json {
        let _ = writeln!(io::stdout().lock(), "{}", error.json());
    } else {
        let _ = write!(io::stderr().lock(), "{}", error.text());
    }
    error.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_covers_quotes_backslashes_and_controls() {
        assert_eq!(escape("a\"b\\c\u{1}\t"), "a\\\"b\\\\c\\u0001\\t");
    }

    #[test]
    fn json_envelope_has_code_and_message() {
        let error = Error::usage("unknown flag '--x'".to_string(), Vec::new());
        assert_eq!(
            error.json(),
            r#"{"error":{"code":"usage","message":"unknown flag '--x'"}}"#
        );
    }
}
