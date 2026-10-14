//! memex library: the module map for the intent-layer CLI, and the run function that main.rs calls.
//! src/README.md says what each module does.
//! Cases: CLI-*, LOG-WRITE-*, SHOW-*, SRCH-*, INFO-NOW-01, SRCH-NOW-01, SHOW-NOW-01

mod atomic;
mod cite;
pub mod cli;
mod clock;
mod collections;
mod config;
mod corpus;
pub mod error;
mod fence;
mod grammar;
pub mod help;
mod index;
mod intent;
mod logs;
mod par;
mod refs;
mod render;
mod resolve;
mod root;
mod scan;
mod text;
mod verbs;
mod walk;

use std::ffi::OsString;
use std::io::{self, Write};

use cli::Command;
use clock::Timestamp;
use error::Error;

/// Runs memex on the arguments after the program name and returns the exit code.
///
/// Help and version print to stdout and exit 0. A usage error exits 2 through `error::report`.
pub fn run(args: Vec<OsString>) -> i32 {
    let json = cli::json_requested(&args);
    match cli::parse(&args) {
        Ok(command) => dispatch(command),
        Err(error) => error::report(&error, json),
    }
}

/// Carries out a parsed command. Each verb starts by reading the clock once, so an invalid `$MEMEX_NOW` exits 3 for every verb
/// before it reads or writes anything (design §3.6); help and version never read it. A search with `--in` and no words is the
/// roster (design §3.2), which the search verb prints.
fn dispatch(command: Command) -> i32 {
    match command {
        Command::Help(verb) => print_out(&help::text(verb)),
        Command::Version => print_out(&format!("memex {}\n", env!("CARGO_PKG_VERSION"))),
        Command::Search {
            terms,
            in_,
            limit,
            json,
        } => {
            let request = verbs::search::Request {
                terms,
                in_,
                limit,
                json,
            };
            finish(started(|now| verbs::search::run(&request, now)), json)
        }
        Command::Show {
            refs,
            up,
            limit,
            json,
        } => {
            let request = verbs::show::Request {
                refs,
                up,
                limit,
                json,
            };
            finish_show(started(|_| verbs::show::run(&request)), json)
        }
        Command::Log {
            stream,
            to,
            tag,
            agent,
            json,
        } => {
            let request = verbs::log::Request {
                stream,
                to,
                tag,
                agent,
                json,
            };
            finish(started(|now| verbs::log::run(&request, now)), json)
        }
        Command::Info { agent, json } => {
            let request = verbs::info::Request { agent, json };
            finish(started(|now| verbs::info::run(&request, now)), json)
        }
        Command::Init => match clock::now() {
            Ok(now) => finish_init(now),
            Err(error) => error::report(&error, false),
        },
    }
}

/// Runs a verb with the instant of the run: `MEMEX_NOW`, else the system clock, read once as the verb starts. `show` reads no
/// time, but its `MEMEX_NOW` is checked all the same.
fn started<T>(verb: impl FnOnce(Timestamp) -> Result<T, Error>) -> Result<T, Error> {
    verb(clock::now()?)
}

/// Prints a verb's stdout, or its error the way the invocation asked for, and returns the exit status.
fn finish(result: Result<String, Error>, json: bool) -> i32 {
    match result {
        Ok(text) => print_out(&text),
        Err(error) => error::report(&error, json),
    }
}

/// Prints a `show` report: its stdout, then its stderr. The exit status is 1 when a ref failed, and a fatal error is
/// reported the usual way.
fn finish_show(result: Result<verbs::show::Report, Error>, json: bool) -> i32 {
    match result {
        Ok(report) => {
            let code = print_out(&report.stdout);
            let _ = write!(io::stderr().lock(), "{}", report.stderr);
            if code == 0 && report.failed {
                1
            } else {
                code
            }
        }
        Err(error) => error::report(&error, json),
    }
}

/// Writes text to stdout and returns the exit status. A closed stdout is silent and exits 0 (design §3.6). Any other failed
/// write exits 3, with the io error named and not its OS text.
fn print_out(text: &str) -> i32 {
    let mut out = io::stdout().lock();
    match stdout_error(write_text(&mut out, text)) {
        None => 0,
        Some(error) => error::report(&error, false),
    }
}

/// Writes `text` and flushes it, and the result is the first io error of either step.
fn write_text(out: &mut impl Write, text: &str) -> io::Result<()> {
    out.write_all(text.as_bytes())?;
    out.flush()
}

/// The error that a stdout write reports, or none. A BrokenPipe means the reader closed stdout early, which is not an error:
/// memex stops writing and says nothing more.
fn stdout_error(written: io::Result<()>) -> Option<Error> {
    match written {
        Ok(()) => None,
        Err(failure) if failure.kind() == io::ErrorKind::BrokenPipe => None,
        Err(failure) => Some(Error::io("cannot write to stdout", &failure)),
    }
}

/// Carries out `init` and prints what it did: the action lines and the atlas to stdout, the warnings to stderr. A failure still
/// prints the actions taken before it, then its error, so the user sees how far init got.
fn finish_init(now: Timestamp) -> i32 {
    let mut report = verbs::init::Report::default();
    let result = verbs::init::run(&mut report, now);
    let code = print_out(&report.stdout);
    let _ = write!(io::stderr().lock(), "{}", report.stderr);
    match result {
        Ok(()) => code,
        Err(error) => error::report(&error, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A writer whose every call fails with one kind, as a stdout that is closed or refused does.
    struct Failing(io::ErrorKind);

    impl Write for Failing {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(self.0))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::from(self.0))
        }
    }

    #[test]
    fn a_closed_stdout_is_not_an_error() {
        let written = write_text(&mut Failing(io::ErrorKind::BrokenPipe), "text\n");
        assert!(stdout_error(written).is_none());
    }

    #[test]
    fn any_other_stdout_failure_is_named_by_its_kind() {
        let written = write_text(&mut Failing(io::ErrorKind::PermissionDenied), "text\n");
        let error = stdout_error(written).expect("a refused write is an error");
        assert_eq!(error.code, error::Code::Io);
        assert_eq!(error.message, "cannot write to stdout (PermissionDenied)");
    }

    #[test]
    fn a_write_that_succeeds_leaves_its_text_and_no_error() {
        let mut sink = Vec::new();
        let written = write_text(&mut sink, "text\n");
        assert!(stdout_error(written).is_none());
        assert_eq!(sink, b"text\n");
    }
}
