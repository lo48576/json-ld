//! JSON-LD syntax related stuff.

/// Checks whether a string has the form of a keyword.
///
/// > having the form of a keyword (i.e., it matches the ABNF rule `"@"1*ALPHA` from \[RFC5234\]),
/// >
/// > --- <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/>
pub(crate) fn has_form_of_keyword(s: &str) -> bool {
    s.len() >= 2 && s.starts_with('@') && s[1..].bytes().all(|b| b.is_ascii_alphabetic())
}
