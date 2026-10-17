//! Adds two numbers for the demo.
//! It has a header of two lines, then code.

fn add(a: i32, b: i32) -> i32 {
    a + b
}

// First block, line one.
// First block, line two.

// First block, line three, after a blank line.
fn x() {} // trailing comment, not read

fn y() {
    // inner
    let _ = 1;
}

/* Third block, a block comment
 * that runs over three lines
 * and closes here. */
fn z() {}
