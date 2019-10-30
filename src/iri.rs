//! IRI-related helpers.

/// IRI category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IriCategory {
    /// Blank node identifier.
    BlankNodeIdentifier,
    /// Absolute IRI (which can contain fragment part).
    AbsoluteIri,
    /// Compact IRI.
    ///
    /// Note that this is also a relative IRI.
    CompactIri,
}

impl IriCategory {
    /// Returns `IriCategory` for the given prefix and suffix.
    fn from_prefix_and_suffix(prefix: &str, suffix: &str) -> Self {
        if prefix == "_" {
            return IriCategory::BlankNodeIdentifier;
        }
        if suffix.starts_with("//") {
            // NOTE: In JSON-LD spec, "absolute IRI" can have fragment part.
            // This is "IRI" but not "absolute IRI" in RFC 3987.
            IriCategory::AbsoluteIri
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
pub(crate) fn to_prefix_and_suffix(s: &str) -> Option<(&str, &str)> {
    s.find(':')
        .map(|colon_pos| (&s[..colon_pos], &s[(colon_pos + 1)..]))
}

/// Checks whether the given string is has the form of an absolute IRI.
pub(crate) fn is_absolute_iri(s: &str) -> bool {
    IriCategory::from(s) == IriCategory::AbsoluteIri
}
