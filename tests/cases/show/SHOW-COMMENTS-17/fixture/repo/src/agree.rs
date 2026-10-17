//! agree.rs: three comment blocks, one with a blank line.
//! It has a two-line header, then code.

pub fn a() -> u32 {
    1
}

// Block one: a single line.
pub fn b() -> u32 {
    2
}

// Block two: a run with a blank line inside it.

// It continues after the blank line.
pub fn c() -> u32 {
    3
}

/* Block three: a block comment
 * over two lines. */
pub fn d() -> u32 {
    4
}
