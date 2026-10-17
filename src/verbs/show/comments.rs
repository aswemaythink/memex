//! Reads a file's comment blocks for `show --comments` (design §3.3 Comments).
//! It also reads the list that a file with no header gives without the flag. A file with a header and no flag is not read here, so
//! its section is what it was before the flag existed.
//! Cases: SHOW-COMMENTS-01, SHOW-COMMENTS-02, SHOW-COMMENTS-03, SHOW-COMMENTS-04, SHOW-COMMENTS-05, SHOW-COMMENTS-06, SHOW-COMMENTS-07, SHOW-COMMENTS-08, SHOW-COMMENTS-09

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use super::section::Options;
use crate::intent::header;
use crate::render::{Comments, Listing};

/// The bytes that a comments list reads of a file, at most (design §3.3): a longer file is cut there, and the list says so.
const READ_LIMIT: u64 = 1024 * 1024;

/// The blocks that `--comments` lists when `--limit` is not given (design §3.3).
const COMMENTS_SHOWN: usize = 20;

/// The blocks that a file with no header lists without the flag, or fewer when a `--limit` is lower (design §3.3).
const FALLBACK_SHOWN: usize = 3;

/// The comment list of the file at `path`, or None when its section lists none (design §3.3 Comments). The list is read when
/// `--comments` is given, or when the file has no header: `has_header` is whether the corpus found one. A file with no comment
/// family, such as Markdown, and a binary file have no comments, so nothing is read for them.
pub fn of_file(path: &Path, has_header: bool, options: &Options) -> io::Result<Option<Comments>> {
    let asked = options.comments;
    if (has_header && !asked) || !header::has_family(path) {
        return Ok(None);
    }
    let (bytes, cut) = read_capped(path)?;
    let Some(blocks) = header::comment_blocks(path, &bytes) else {
        return Ok(None);
    };
    if blocks.is_empty() && !asked {
        return Ok(None);
    }
    let shown = shown(asked, options.limit_given);
    Ok(Some(Comments {
        list: Listing::of(&blocks, shown),
        cut,
        asked,
    }))
}

/// How many blocks a list shows: `--comments` shows its `--limit`, or 20; a file with no header shows 3, or its `--limit` when that is
/// lower (design §3.3 Comments).
fn shown(asked: bool, limit: Option<usize>) -> usize {
    match (asked, limit) {
        (true, limit) => limit.unwrap_or(COMMENTS_SHOWN),
        (false, limit) => limit.map_or(FALLBACK_SHOWN, |limit| limit.min(FALLBACK_SHOWN)),
    }
}

/// The bytes of the file at `path`, up to `READ_LIMIT`, and whether the file is longer than they are. When it is cut, the last line
/// that the read did not finish is left out, so that no block ends in half a line.
fn read_capped(path: &Path) -> io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::new();
    File::open(path)?.take(READ_LIMIT).read_to_end(&mut bytes)?;
    let cut = fs::metadata(path).is_ok_and(|meta| meta.len() > READ_LIMIT);
    if cut {
        let whole = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |end| end + 1);
        bytes.truncate(whole);
    }
    Ok((bytes, cut))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flag_shows_its_limit_or_twenty_and_no_flag_shows_three_at_most() {
        assert_eq!(shown(true, None), 20);
        assert_eq!(shown(true, Some(2)), 2);
        assert_eq!(shown(false, None), 3);
        assert_eq!(shown(false, Some(2)), 2);
        assert_eq!(shown(false, Some(9)), 3);
    }

    #[test]
    fn a_cut_read_keeps_only_whole_lines() {
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("long.rs");
        let mut body = b"// first\n".to_vec();
        body.extend(std::iter::repeat_n(b'x', READ_LIMIT as usize));
        fs::write(&path, &body).expect("a written file");
        let (bytes, cut) = read_capped(&path).expect("a read");
        assert!(cut);
        assert_eq!(bytes, b"// first\n");
    }
}
