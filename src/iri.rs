//! IRI-related helpers.

/// IRI category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IriCategory {
    /// Blank node identifier.
    BlankNodeIdentifier,
    /// Absolute IRI reference (which can contain fragment part).
    ///
    /// Note that this is "IRI" in RFC 3987, but not "absolute IRI".
    AbsoluteIriRef,
    /// Compact IRI.
    ///
    /// Note that this is also a relative IRI reference.
    CompactIri,
}

impl IriCategory {
    /// Returns `IriCategory` for the given prefix and suffix.
    fn from_prefix_and_suffix(prefix: &str, suffix: &str) -> Self {
        if prefix == "_" {
            return IriCategory::BlankNodeIdentifier;
        }
        if suffix.starts_with("//") {
            IriCategory::AbsoluteIriRef
        } else {
            IriCategory::CompactIri
        }
    }
}

impl From<&str> for IriCategory {
    fn from(s: &str) -> Self {
        to_prefix_and_suffix(s).map_or(IriCategory::CompactIri, |(prefix, suffix)| {
            Self::from_prefix_and_suffix(prefix, suffix)
        })
    }
}

/// Split the given string to prefix part and suffix part.
///
/// Prefix part to be returned is not be empty.
pub(crate) fn to_prefix_and_suffix(s: &str) -> Option<(&str, &str)> {
    if s.is_empty() {
        return None;
    }
    // The first character should be treated as normal character rather than splitting character.
    // See <https://github.com/w3c/json-ld-api/issues/189> and
    // <https://github.com/w3c/json-ld-api/pull/203>.
    s[1..].find(':').map(|before_colon_pos| {
        assert_eq!(s.as_bytes()[before_colon_pos + 1], b':');
        (&s[..=before_colon_pos], &s[(before_colon_pos + 2)..])
    })
}

/// Checks whether the given string is has the form of a compact IRI (or relative IRI reference).
pub(crate) fn is_compact_iri(s: &str) -> bool {
    IriCategory::from(s) == IriCategory::CompactIri
}

/// Checks whether the given string is has the form of an IRI (absolute form).
pub(crate) fn is_absolute_iri_ref(s: &str) -> bool {
    IriCategory::from(s) == IriCategory::AbsoluteIriRef
}

/// Checks whether the given string is has the form of an IRI (absolute form).
pub(crate) fn is_absolute_ref_or_blank_node_ident(s: &str) -> bool {
    match IriCategory::from(s) {
        IriCategory::AbsoluteIriRef | IriCategory::BlankNodeIdentifier => true,
        _ => false,
    }
}

/// Checks is the given ASCII byte is `gen-delims` character.
pub(crate) fn is_gen_delims_byte(b: u8) -> bool {
    match b {
        b':' | b'/' | b'?' | b'#' | b'[' | b']' | b'@' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_prefix() {
        assert_eq!(to_prefix_and_suffix("foo"), None);
        assert_eq!(to_prefix_and_suffix("foo:bar"), Some(("foo", "bar")));
        assert_eq!(to_prefix_and_suffix(":foo"), None);
        assert_eq!(to_prefix_and_suffix("foo:"), Some(("foo", "")));
        assert_eq!(to_prefix_and_suffix(":foo:"), Some((":foo", "")));
        assert_eq!(to_prefix_and_suffix(":foo:bar:"), Some((":foo", "bar:")));
    }
}
