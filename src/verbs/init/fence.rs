//! Finds the lines outside fenced code blocks (design §2.3). Init looks for its markers and import only there.
//! Cases: INIT-FENCE-01, INIT-FENCE-02, INIT-FENCE-03, INIT-FENCE-04, INIT-FENCE-05, INIT-FENCE-06, INIT-FENCE-07

/// What the fences of a text leave outside them.
#[derive(Debug, PartialEq, Eq)]
pub struct Outside {
    /// For each line, true when the line is outside every fence. A fence's opening and closing lines are inside it.
    pub live: Vec<bool>,
    /// True when the last fence is never closed, so it runs to the end of the text.
    pub open_at_end: bool,
}

/// A fence that is open: the mark it opened with, and how many marks it has at least.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fence {
    mark: u8,
    len: usize,
}

/// Scans the lines of a text for fenced code blocks. Each line may keep its line ending. A fence opens with three or more
/// backticks or tildes, indented at most three spaces, and closes at a line of the same mark, at least as long, indented at
/// most three spaces, with nothing else on it (design §2.3).
pub fn scan(lines: &[&str]) -> Outside {
    let mut open: Option<Fence> = None;
    let mut live = Vec::with_capacity(lines.len());
    for line in lines {
        let text = line.trim_end_matches(['\r', '\n']);
        match open {
            Some(fence) => {
                live.push(false);
                if closes(text, fence) {
                    open = None;
                }
            }
            None => {
                open = opens(text);
                live.push(open.is_none());
            }
        }
    }
    Outside {
        live,
        open_at_end: open.is_some(),
    }
}

/// The fence that a line opens, if it does. A backtick fence's info string may not hold a backtick, so such a line is text.
fn opens(text: &str) -> Option<Fence> {
    let body = indented(text)?;
    let mark = *body.as_bytes().first()?;
    if mark != b'`' && mark != b'~' {
        return None;
    }
    let len = run(body, mark);
    let info = &body[len..];
    (len >= 3 && !(mark == b'`' && info.contains('`'))).then_some(Fence { mark, len })
}

/// True for a line that closes `fence`: the same mark at least as many times, indented at most three spaces, and nothing else.
fn closes(text: &str, fence: Fence) -> bool {
    let Some(body) = indented(text) else {
        return false;
    };
    let len = run(body, fence.mark);
    len >= fence.len && body[len..].trim().is_empty()
}

/// The line after its leading spaces, when there are at most three of them. A line indented further is code, not a fence.
fn indented(text: &str) -> Option<&str> {
    let body = text.trim_start_matches(' ');
    (text.len() - body.len() <= 3).then_some(body)
}

/// The number of bytes equal to `mark` at the start of `text`.
fn run(text: &str, mark: u8) -> usize {
    text.bytes().take_while(|byte| *byte == mark).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The live flags and the open-at-end flag of a text, scanned as its lines with their endings.
    fn scanned(text: &str) -> (Vec<bool>, bool) {
        let lines: Vec<&str> = text.split_inclusive('\n').collect();
        let outside = scan(&lines);
        (outside.live, outside.open_at_end)
    }

    #[test]
    fn a_fence_hides_its_lines_and_its_own_marks() {
        let (live, open) = scanned("# A\n```md\nx\n```\ny\n");
        assert_eq!(live, [true, false, false, false, true]);
        assert!(!open);
    }

    #[test]
    fn a_fence_left_open_runs_to_the_end_of_the_text() {
        let (live, open) = scanned("a\n```\nb\nc\n");
        assert_eq!(live, [true, false, false, false]);
        assert!(open);
    }

    #[test]
    fn a_closing_line_needs_the_same_mark_at_least_as_long() {
        let (live, open) = scanned("~~~\n```\n~~~~\nx\n");
        assert_eq!(live, [false, false, false, true]);
        assert!(!open);
        let (live, open) = scanned("````\n```\nx\n");
        assert_eq!(live, [false, false, false]);
        assert!(open);
    }

    #[test]
    fn a_closing_line_with_other_text_does_not_close() {
        let (live, open) = scanned("```\n```md\nx\n");
        assert_eq!(live, [false, false, false]);
        assert!(open);
    }

    #[test]
    fn a_fence_indented_more_than_three_spaces_is_text() {
        let (live, open) = scanned("    ```\nx\n");
        assert_eq!(live, [true, true]);
        assert!(!open);
    }

    #[test]
    fn a_backtick_info_string_with_a_backtick_is_text() {
        let (live, open) = scanned("```a`b\nx\n");
        assert_eq!(live, [true, true]);
        assert!(!open);
    }
}
