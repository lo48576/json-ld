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
/// See <https://www.w3.org/TR/2019/WD-json-ld11-20191112/#dfn-term-definition> and
/// <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#context-processing-algorithm>.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Definition {
    /// IRI mapping or reverse property.
    // This can be a non-IRI-reference (such as keywords), so use `String` here.
    // TODO: This is an IRI (including a blank node identifier) or a keyword.
    iri: String,
    /// Reverse property flag.
    reverse: bool,
    /// Type mapping (optional).
    // TODO: This is an IRI.
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

    /// Compares the term definitions other than `@protected` flag).
    pub(crate) fn eq_other_than_protected(&self, other: &Self) -> bool {
        self.iri == other.iri
            && self.reverse == other.reverse
            && self.ty == other.ty
            && self.language == other.language
            && self.direction == other.direction
            && self.context == other.context
            && self.nest == other.nest
            && self.prefix == other.prefix
            && self.index == other.index
            && self.protected == other.protected
            && self.container == other.container
    }
}
