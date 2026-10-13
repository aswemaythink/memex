//! memex library: the module map for the intent-layer CLI, and the run function that main.rs calls.
//! Modules are listed in PLAN.md §4 and added slice by slice.
//! Cases: CLI-*, LOG-WRITE-*

pub mod cli;
mod clock;
mod config;
pub mod error;
mod grammar;
pub mod help;
mod logs;
mod root;
mod scan;
mod verbs;

use std::ffi::OsString;
use std::io::{self, Write};

use cli::{Command, Verb};
use error::Error;

/// Runs memex on the arguments after the program name and returns the exit code.
///
/// Help and version print to stdout and exit 0. A usage error exits 2 through `error::report`.
/// A verb that parses is not implemented yet, so it exits 2 with a line on stderr.
pub fn run(args: Vec<OsString>) -> i32 {
    let json = cli::json_requested(&args);
    match cli::parse(&args) {
        Ok(command) => dispatch(command),
        Err(error) => error::report(&error, json),
    }
}

/// Carries out a parsed command. Help, version and log do their work; the other verbs are not built yet.
fn dispatch(command: Command) -> i32 {
    match command {
        Command::Help(verb) => print_out(&help::text(verb)),
        Command::Version => print_out(&format!("memex {}\n", env!("CARGO_PKG_VERSION"))),
        Command::Search { .. } => not_implemented(Verb::Search),
        Command::Show { .. } => not_implemented(Verb::Show),
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
            finish(verbs::log::run(&request), json)
        }
        Command::Info { .. } => not_implemented(Verb::Info),
        Command::Init => not_implemented(Verb::Init),
    }
}

/// Prints a verb's stdout, or its error the way the invocation asked for, and returns the exit status.
fn finish(result: Result<String, Error>, json: bool) -> i32 {
    match result {
        Ok(text) => print_out(&text),
        Err(error) => error::report(&error, json),
    }
}

/// Writes text to stdout. A failed write exits 3, with the io error named and not its OS text.
fn print_out(text: &str) -> i32 {
    let mut out = io::stdout().lock();
    let written = out.write_all(text.as_bytes());
    let written = written.and_then(|()| out.flush());
    match written {
        Ok(()) => 0,
        Err(failure) => error::report(&Error::io("cannot write to stdout", &failure), false),
    }
}

/// Reports a verb that parsed but has no implementation yet. No case pins this line.
fn not_implemented(verb: Verb) -> i32 {
    let _ = writeln!(
        io::stderr().lock(),
        "memex: {} is not implemented yet",
        verb.name()
    );
    2
}
