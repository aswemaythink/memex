//! twin.rs: the same file with LF or CRLF line ends.
//! It has a two-line header, then code.

fn one() -> u32 {
    1
}

// First block: a run with a blank line inside it.

// It continues here.
fn two() -> u32 {
    2
}

/* Third block: a block comment
 * over two lines. */
fn three() -> u32 {
    3
}
