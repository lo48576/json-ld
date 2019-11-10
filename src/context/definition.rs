//! Term definition.

use crate::{context::Context, json::Nullable};

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
/// See <https://www.w3.org/TR/2019/WD-json-ld11-20191018/#dfn-term-definition> and
/// <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#context-processing-algorithm>.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Definition {
    /// IRI mapping or reverse property.
    // This can be a non-IRI-reference (such as keywords), so use `String` here.
    iri: String,
    /// Reverse property flag.
    reverse: bool,
    /// Type mapping (optional).
    ty: Option<String>,
    /// Lanugage mapping (optional).
    language: Option<Nullable<String>>,
    /// Direction mapping (optional).
    direction: Option<Direction>,
    /// Context (optional).
    context: Option<Context>,
    /// Nest value (optional).
    nest: Option<String>,
    /// Prefix flag (optoinal).
    prefix: Option<bool>,
    /// Index mapping (optional).
    index: Option<String>,
    /// "Protected" flag (optional).
    protected: Option<bool>,
    /// Container mapping (optional).
    container: Option<Container>,
}

impl Definition {
    /// Returns the IRI mapping.
    pub(crate) fn iri(&self) -> &str {
        &self.iri
    }

    /// Returns the prefix flag.
    pub(crate) fn is_prefix(&self) -> bool {
        self.prefix.unwrap_or(false)
    }

    /// Returns whether the definition is protected.
    ///
    /// Returns false if the value is not set.
    pub(crate) fn is_protected(&self) -> bool {
        self.protected.unwrap_or(false)
    }
}
