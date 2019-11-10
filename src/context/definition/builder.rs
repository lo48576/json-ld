//! Definition builder.

use crate::{
    context::{
        definition::{Container, ContainerItem, Direction},
        Context, Definition,
    },
    json::Nullable,
};

/// Builder of `Definition`.
#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct DefinitionBuilder {
    /// IRI mapping or reverse property.
    // This can be a non-IRI-reference (such as keywords), so use `String` here.
    iri: Option<String>,
    /// Reverse property flag.
    reverse: Option<bool>,
    /// Type mapping (optional).
    ty: Option<String>,
    /// Lanugage mapping (optional).
    ///
    /// This property distinguishes explicit `null`.
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
            reverse: self.reverse.expect(
                "Reverse property flag must be explicitly set by create term definition algorithm",
            ),
            ty: self.ty,
            language: self.language,
            direction: self.direction,
            context: self.context,
            nest: self.nest,
            prefix: self.prefix,
            index: self.index,
            protected: self.protected,
            container: self.container,
        }
    }

    /// Sets the IRI mapping.
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

    /// Sets the reverse property flag.
    pub(crate) fn set_reverse(&mut self, v: bool) {
        self.reverse = Some(v);
    }

    /// Sets the type mapping.
    pub(crate) fn set_ty(&mut self, v: impl Into<String>) {
        self.ty = Some(v.into());
    }

    /// Returns the type mapping.
    pub(crate) fn ty(&self) -> Option<&str> {
        self.ty.as_ref().map(AsRef::as_ref)
    }

    /// Sets the language mapping.
    pub(crate) fn set_language(&mut self, v: impl Into<Nullable<String>>) {
        self.language = Some(v.into())
    }

    /// Sets the direction mapping.
    pub(crate) fn set_direction(&mut self, v: Nullable<Direction>) {
        self.direction = v.into();
    }

    /// Sets the local context.
    pub(crate) fn set_local_context(&mut self, v: Context) {
        self.context = Some(v);
    }

    /// Sets the nest value.
    pub(crate) fn set_nest(&mut self, v: impl Into<String>) {
        self.nest = Some(v.into())
    }

    /// Sets the prefix flag.
    pub(crate) fn set_prefix(&mut self, v: bool) {
        self.prefix = Some(v);
    }

    /// Sets the index mapping.
    pub(crate) fn set_index(&mut self, v: impl Into<String>) {
        self.index = Some(v.into());
    }

    /// Sets the "protected" flag.
    pub(crate) fn set_protected(&mut self, v: bool) {
        self.protected = Some(v);
    }

    /// Checks if a definition to be built is same as the given definition other than the value of
    /// the protected flag.
    pub(crate) fn is_same_other_than_protected(&self, _other: &Definition) -> bool {
        unimplemented!("Compare definitions")
    }

    /// Sets the container mapping.
    pub(crate) fn set_container(&mut self, v: Nullable<Container>) {
        self.container = v.into();
    }

    /// Checks if the `@conatiner` value contains the given value.
    ///
    /// Returns `false` if the container mapping is not set.
    pub(crate) fn container_contains(&self, v: ContainerItem) -> bool {
        self.container
            .as_ref()
            .map_or(false, |container| container.contains(v))
    }

    /// Returns the container mapping.
    pub(crate) fn container(&self) -> Option<&Container> {
        self.container.as_ref()
    }
}
