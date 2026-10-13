//! Binary entry point: turns memex::run's result into the process exit code.

fn main() {
    std::process::exit(memex::run(std::env::args_os().skip(1).collect()));
}
