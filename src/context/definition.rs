//! Term definition.

pub(crate) use self::{
    builder::DefinitionBuilder,
    container::{Container, ContainerItem},
    direction::Direction,
};

mod builder;
mod container;
mod direction;

/// Term definition.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-20191018/#dfn-term-definition>.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Definition {
    /// IRI mapping or reverse property.
    // This can be a non-IRI-reference (such as keywords), so use `String` here.
    iri: String,
    /// Reverse property flag.
    reverse: bool,
    /// Prefix flag.
    prefix: bool,
}

impl Definition {
    /// Returns the IRI mapping.
    pub(crate) fn iri(&self) -> &str {
        &self.iri
    }

    /// Returns the prefix flag.
    pub(crate) fn is_prefix(&self) -> bool {
        self.prefix
    }

    /// Returns whether the definition is protected.
    pub(crate) fn is_protected(&self) -> bool {
        unimplemented!()
    }
}
