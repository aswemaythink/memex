//! memex library: the module map for the intent-layer CLI.
//! Modules are listed in PLAN.md §4 and added slice by slice.
//! Cases: CLI-VERSION-01

/// Runs memex on the arguments after the program name and returns the exit code.
///
/// Stub until S0.3 replaces it with the real grammar: `--version` prints the version, and other arguments do nothing.
pub fn run(args: Vec<std::ffi::OsString>) -> i32 {
    if args == [std::ffi::OsString::from("--version")] {
        println!("memex {}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    0
}
