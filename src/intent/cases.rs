//! Reads a CASES.md into case docs (design §2.2). S2.3 fills it in; until then a CASES.md yields no cases.
//! Cases: none (stub; the CASE family, S2.3, pins it)

use crate::corpus::Doc;

/// The case docs of the CASES.md at `rel`, whose text is `text`. None are read yet.
pub fn parse(_rel: &str, _text: &str) -> Vec<Doc> {
    Vec::new()
}
