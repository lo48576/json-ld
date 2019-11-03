//! Definition builder.

use crate::{
    context::{
        definition::{Container, ContainerItem, Direction},
        Context, Definition,
    },
    json::Nullable,
};

/// Builder of `Definition`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DefinitionBuilder {
    /// IRI mapping or reverse property.
    // This can be a non-IRI-reference (such as keywords), so use `String` here.
    iri: Option<String>,
    /// Reverse property flag.
    reverse: Option<bool>,
    /// Prefix flag.
    prefix: Option<bool>,
    /// Container mapping.
    container: Option<Nullable<Container>>,
}

impl DefinitionBuilder {
    /// Creates a new builder.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Builds a definition.
    ///
    /// # Panics
    ///
    /// Panics if the necessary fields are not set.
    pub(crate) fn build(self) -> Definition {
        Definition {
            iri: self.iri.expect("IRI mapping must be set"),
            reverse: self
                .reverse
                .expect("reverse property flag must be explicitly set"),
            prefix: self
                .prefix
                .expect("FIXME: Should prefix flag must be explicitly set?"),
        }
    }

    /// Sets "IRI".
    pub(crate) fn set_iri(&mut self, v: impl Into<String>) {
        self.iri = Some(v.into());
    }

    /// Returns the IRI mapping.
    ///
    /// # Panics
    ///
    /// Panics if the IRI mapping is not set.
    pub(crate) fn iri(&self) -> &str {
        self.iri.as_ref().expect("IRI mapping must be set").as_str()
    }

    /// Sets "type".
    pub(crate) fn set_ty(&mut self, _: impl Into<String>) {
        unimplemented!()
    }

    /// Returns the type mapping.
    pub(crate) fn ty(&self) -> Option<&str> {
        unimplemented!()
    }

    /// Sets "container".
    pub(crate) fn set_container(&mut self, v: Nullable<Container>) {
        self.container = Some(v);
    }

    /// Checks if the `@conatiner` value contains the given value.
    ///
    /// Returns `false` if the container mapping is not set.
    pub(crate) fn container_contains(&self, v: ContainerItem) -> bool {
        match &self.container {
            None | Some(Nullable::Null) => false,
            Some(Nullable::Value(container)) => container.contains(v),
        }
    }

    /// Returns the container mapping.
    pub(crate) fn container(&self) -> Option<&Container> {
        unimplemented!()
    }

    /// Sets reverse property flag.
    pub(crate) fn set_reverse(&mut self, v: bool) {
        self.reverse = Some(v);
    }

    /// Sets "protected".
    pub(crate) fn set_protected(&mut self, _: bool) {
        unimplemented!()
    }

    /// Sets "prefix".
    pub(crate) fn set_prefix(&mut self, v: bool) {
        self.prefix = Some(v);
    }

    /// Sets "index".
    pub(crate) fn set_index(&mut self, _: impl Into<String>) {
        unimplemented!()
    }

    /// Sets "nest".
    pub(crate) fn set_nest(&mut self, _: impl Into<String>) {
        unimplemented!()
    }

    /// Sets "local context".
    pub(crate) fn set_local_context(&mut self, _: Context) {
        unimplemented!()
    }

    /// Sets "language".
    pub(crate) fn set_language(&mut self, _: impl Into<Nullable<String>>) {
        unimplemented!()
    }

    /// Sets "direction".
    pub(crate) fn set_direction(&mut self, _: Nullable<Direction>) {
        unimplemented!()
    }

    /// Checks if a definition to be built is same as the given definition other than the value of
    /// the protected flag.
    pub(crate) fn is_same_other_than_protected(&self, _other: &Definition) -> bool {
        unimplemented!()
    }
}
