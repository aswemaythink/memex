//! The spellings that refs in docs use (design §2.2, §2.3): case ids, path tokens and the path join.
//! The CASES.md reader and the citation scanner both read them, so each rule has one copy.
//! Cases: CASE-GRAMMAR-01, CASE-GRAMMAR-02, CASE-GRAMMAR-03, CASE-EVID-05, CITE-BARE-01, CITE-BARE-02, CITE-PATH-05, CITE-PATH-06

/// True for an id body (design §2.2): a capital first, then dash-separated parts of capitals and digits, none of them empty.
/// A family prefix such as `NAME-` is an id body followed by a dash (design §2.3).
pub fn is_id_body(body: &str) -> bool {
    body.starts_with(|c: char| c.is_ascii_uppercase())
        && body.split('-').all(is_capitals_and_digits)
}

/// True for a case id (design §2.2): an id body with at least one dash, so at least two parts.
pub fn is_case_id(id: &str) -> bool {
    id.contains('-') && is_id_body(id)
}

/// True for a path token's extension: the last dot is followed by one or more letters or digits (design §2.3).
pub fn has_extension(token: &str) -> bool {
    token.rsplit_once('.').is_some_and(|(_, ext)| {
        !ext.is_empty() && ext.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
}

/// The root-relative path that `token` names from the folder `base`: `.` parts are dropped and each `..` climbs one folder.
/// None when a `..` would climb above the root (design §2.3).
pub fn join(base: &str, token: &str) -> Option<String> {
    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    for part in token.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name),
        }
    }
    Some(parts.join("/"))
}

/// True for a non-empty run of upper-case ASCII letters and digits.
fn is_capitals_and_digits(part: &str) -> bool {
    !part.is_empty()
        && part
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_case_id_has_capitals_and_digits_in_at_least_two_parts() {
        assert!(is_case_id("NAME-001"));
        assert!(is_case_id("A-1"));
        assert!(is_case_id("LIB-MEASURE-001"));
        assert!(!is_case_id("name-001"));
        assert!(!is_case_id("NODASH"));
        assert!(!is_case_id("TRAILING-"));
        assert!(!is_case_id("DOUBLE--1"));
    }

    #[test]
    fn a_family_prefix_is_an_id_body_without_its_dash() {
        assert!(is_id_body("NAME"));
        assert!(!is_id_body("name"));
        assert!(!is_id_body("NAME-"));
    }

    #[test]
    fn an_extension_is_a_dot_then_letters_or_digits() {
        assert!(has_extension("cli.rs"));
        assert!(!has_extension("README"));
        assert!(!has_extension("a."));
    }

    #[test]
    fn a_join_stays_inside_the_root() {
        assert_eq!(join("src/deep", "../cli.rs").as_deref(), Some("src/cli.rs"));
        assert_eq!(join("", "./a/./b/").as_deref(), Some("a/b"));
        assert_eq!(join("", "../x"), None);
        assert_eq!(join("a", "../../x"), None);
    }
}
