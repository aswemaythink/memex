//! Counts a file's lines for `check` (design §3.5d): its whole text, and its header and license blocks.

use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom};
use std::ops::Range;
use std::path::Path;

use super::families::{family_of, Family};
use super::scan;
use super::stream::count_lines;
use crate::text;

/// The line counts of one file with a comment family (design §3.5d): its whole text's lines, and the lines of its header block and
/// of its license block, which the head of the file holds (design §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// The lines of the whole file, read past its head when the file is longer than the head.
    pub total: u32,
    /// The lines of the header block: 0 when the head shows no header.
    pub header: u32,
    /// The lines of the license block: 0 when the head has none.
    pub license: u32,
}

impl Counts {
    /// The code lines (design §3.5d Kinds, item 3): the lines that are neither header nor license lines. Blank lines and other
    /// comments are code lines.
    pub fn code(&self) -> u32 {
        self.total
            .saturating_sub(self.header)
            .saturating_sub(self.license)
    }
}

/// The line counts of the file at `path`, whose first bytes are `head` (design §3.5d). None for a file with no comment family, which
/// is not counted. A file no longer than `head` is counted from `head` alone; a longer one is read after `head`, which is not read again.
pub fn counts(path: &Path, head: &[u8]) -> io::Result<Option<Counts>> {
    let Some(family) = family_of(path) else {
        return Ok(None);
    };
    let total = total_lines(path, head)?;
    let (header, license) = block_lines(family, head);
    Ok(Some(Counts {
        total,
        header,
        license,
    }))
}

/// The lines of the whole file at `path`, whose first bytes are `head`: `head` alone when it is the whole file, else `head` and the
/// rest of the file after it.
fn total_lines(path: &Path, head: &[u8]) -> io::Result<u32> {
    let head_len = head.len() as u64;
    if fs::metadata(path)?.len() <= head_len {
        return count_lines(head, io::empty());
    }
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(head_len))?;
    count_lines(head, file)
}

/// The header and license line counts that the header scan finds in `head` (design §2.1), as (header, license). A binary head has
/// neither, since the header reader skips a binary file.
fn block_lines(family: &Family, head: &[u8]) -> (u32, u32) {
    if head.contains(&0) {
        return (0, 0);
    }
    let text = text::decode(head);
    let lines: Vec<&str> = text.lines().collect();
    let (_, license, header) = scan::scan(family, &lines);
    (span_len(header), span_len(license))
}

/// The number of lines in `span`, which no real file has more of than a u32 holds.
fn span_len(span: Range<usize>) -> u32 {
    u32::try_from(span.len()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// The size of the head that header detection reads from a file (design §2.1).
    const HEAD: usize = 8 * 1024;

    /// The counts of `text` as a file named `name`, with its first 8 KiB as the head, as `check` reads it.
    fn counts_of(name: &str, text: &[u8]) -> Counts {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join(name);
        fs::write(&path, text).expect("a written file");
        let head = &text[..text.len().min(HEAD)];
        counts(&path, head)
            .expect("counts")
            .expect("a comment family")
    }

    /// The total, header and license line counts of `got`, in that order.
    fn tally(got: Counts) -> (u32, u32, u32) {
        (got.total, got.header, got.license)
    }

    #[test]
    fn a_file_with_no_comment_family_has_no_counts() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("data.bin");
        fs::write(&path, "x\n").expect("a written file");
        assert_eq!(counts(&path, b"x\n").expect("counts"), None);
    }

    #[test]
    fn an_empty_file_with_a_family_counts_nothing() {
        assert_eq!(tally(counts_of("empty.rs", b"")), (0, 0, 0));
    }

    #[test]
    fn a_header_with_no_license_counts_no_license_lines() {
        let got = counts_of("lib.rs", b"//! Purpose.\n//! More.\n\nfn x() {}\n");
        assert_eq!(tally(got), (4, 2, 0));
        assert_eq!(got.code(), 2);
    }

    #[test]
    fn a_shebang_pragma_license_blank_header_and_code_are_told_apart() {
        let text: &[u8] = b"#!/usr/bin/env python3\n\
            # -*- coding: utf-8 -*-\n\
            # SPDX-License-Identifier: MIT\n\
            \n\
            # Purpose line.\n\
            # More.\n\
            \n\
            import os\n";
        let got = counts_of("tool.py", text);
        assert_eq!(tally(got), (8, 2, 1));
        assert_eq!(got.code(), 5);
    }

    #[test]
    fn a_license_is_one_block_up_to_a_blank_line_and_a_header_follows_only_after_one() {
        let joined = counts_of(
            "lib.rs",
            b"// Copyright 2024 Ada\n// Purpose.\n\nfn x() {}\n",
        );
        assert_eq!(tally(joined), (4, 0, 2));
        let separated = counts_of(
            "lib.rs",
            b"// Copyright 2024 Ada\n\n// Purpose.\nfn x() {}\n",
        );
        assert_eq!(tally(separated), (4, 1, 1));
    }

    #[test]
    fn a_file_with_no_header_has_no_header_lines() {
        assert_eq!(
            tally(counts_of("main.rs", b"fn main() {}\n// trailing\n")),
            (2, 0, 0)
        );
        assert_eq!(
            tally(counts_of("lib.rs", b"//\n//\nfn x() {}\n")),
            (3, 0, 0)
        );
    }

    #[test]
    fn a_python_docstring_is_the_header_when_no_hash_block_leads() {
        let text: &[u8] =
            b"#!/usr/bin/env python3\n\"\"\"Purpose.\n\nMore text.\n\"\"\"\n\nimport os\n";
        let got = counts_of("tool.py", text);
        assert_eq!(tally(got), (7, 4, 0));
        assert_eq!(got.code(), 3);
    }

    #[test]
    fn a_header_cut_at_the_read_limit_counts_its_lines_in_the_head() {
        let text = "// x\n".repeat(10_000);
        assert_eq!(
            tally(counts_of("long.rs", text.as_bytes())),
            (10_000, 1_639, 0)
        );
    }

    #[test]
    fn a_file_longer_than_the_head_is_counted_past_it() {
        let text = format!("//! Purpose.\n{}", "fn x() {}\n".repeat(3_000));
        assert_eq!(tally(counts_of("long.rs", text.as_bytes())), (3_001, 1, 0));
    }

    #[test]
    fn a_head_as_long_as_the_file_is_the_whole_file() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("short.rs");
        fs::write(&path, "a\nb\n").expect("a written file");
        let got = counts(&path, b"a\nb\nc\nd\ne\n")
            .expect("counts")
            .expect("a family");
        assert_eq!(got.total, 5, "the head is used, not the file");
    }

    #[test]
    fn a_binary_file_has_no_header_or_license_lines_but_its_lines_count() {
        assert_eq!(
            tally(counts_of("blob.rs", b"//! Purpose.\n\0\n")),
            (2, 0, 0)
        );
    }
}
