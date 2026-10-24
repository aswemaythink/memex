//! Cuts a one-liner for text output (design §2 One-liners).
//! It cuts at 120 characters, ellipsis included, on a whole word where it can.
//! Every text rendering of a one-liner goes through `clip`; `--json` never does.
//! Cases: INT-LINE-05, INT-CLIP-01, INT-CLIP-02, INT-CLIP-03, INT-CLIP-04, INT-CLIP-05

/// A one-liner is cut to this many characters, the ellipsis included (design §2).
const CLIP: usize = 120;

/// The fewest characters a cut at a space must keep; a space earlier than this is passed over and the cut is mid-word.
const MIN_KEPT: usize = 60;

/// A one-liner cut to `CLIP` characters when it is longer. It keeps the first `CLIP - 1` characters, or those up to the last
/// whitespace among them when that leaves at least `MIN_KEPT`, so a line ends on a whole word. The whitespace and any `,`, `;` or
/// `:` at the end of what is kept are dropped before the `…`.
pub fn clip(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= CLIP {
        return text.to_string();
    }
    let room = CLIP - 1;
    let mut end = room;
    if !chars[room].is_whitespace() {
        let space = chars[..room].iter().rposition(|ch| ch.is_whitespace());
        end = space.filter(|at| *at >= MIN_KEPT).unwrap_or(room);
    }
    let kept: String = chars[..end].iter().collect();
    let kept =
        kept.trim_end_matches(|ch: char| ch.is_whitespace() || matches!(ch, ',' | ';' | ':'));
    format!("{kept}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_one_liner_is_whole() {
        assert_eq!(clip("Short."), "Short.");
        assert_eq!(clip(&"a".repeat(120)), "a".repeat(120));
    }

    #[test]
    fn a_cut_lands_on_the_last_whole_word() {
        let words = ["abcdefgh"; 20].join(" ");
        assert_eq!(clip(&words), format!("{}…", ["abcdefgh"; 13].join(" ")));
    }

    #[test]
    fn a_space_right_after_the_room_keeps_the_whole_last_word() {
        let text = format!("{} {} z", "a".repeat(100), "b".repeat(18));
        assert_eq!(
            clip(&text),
            format!("{} {}…", "a".repeat(100), "b".repeat(18))
        );
    }

    #[test]
    fn trailing_punctuation_is_dropped_before_the_ellipsis() {
        let text = format!("{}, {}", "a".repeat(70), "b".repeat(80));
        assert_eq!(clip(&text), format!("{}…", "a".repeat(70)));
    }

    #[test]
    fn a_long_single_word_or_an_early_space_is_cut_mid_word() {
        assert_eq!(clip(&"a".repeat(200)), format!("{}…", "a".repeat(119)));
        let early = format!("ab {}", "c".repeat(200));
        assert_eq!(clip(&early), format!("ab {}…", "c".repeat(116)));
    }
}
