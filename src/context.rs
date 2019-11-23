//! JSON-LD context.
//!
//! See <https://www.w3.org/TR/2019/WD-json-ld11-20191018/#the-context>.

use std::collections::HashMap;

use iri_string::types::{IriStr, IriString};
use serde_json::{Map as JsonMap, Value};

use crate::{error::Result, json::Nullable, processor::Processor, remote::LoadRemoteDocument};

pub(crate) use self::definition::Definition;
use self::{
    create_term_def::{create_term_definition, OptionalParams as CreateTermDefOptionalParams},
    merge::OptionalParams as MergeOptionalParams,
};

mod create_term_def;
mod definition;
mod merge;

/// JSON-LD context.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-20191018/#the-context>.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Context {
    /// Term definitions.
    term_definitions: HashMap<String, Nullable<Definition>>,
    /// Base IRI.
    base: Nullable<IriString>,
    /// Default base direction (optional).
    default_base_direction: Option<definition::Direction>,
    /// Default language (optional).
    default_language: Option<String>,
    /// Context propagation.
    propagate: (),
    /// Vocabulary mapping (optional).
    vocab: Nullable<String>,
    /// Previous context (optional).
    previous_context: Option<Box<Self>>,
}

impl Context {
    /// Creates a new empty `Context`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new `Context` with the given base IRI.
    pub fn with_base(base: IriString) -> Self {
        Self {
            base: Nullable::Value(base),
            ..Default::default()
        }
    }

    /// Returns the base IRI.
    pub(crate) fn base(&self) -> Nullable<&IriStr> {
        self.base.as_ref().map(AsRef::as_ref)
    }

    /// Sets the base IRI.
    pub(crate) fn set_base(&mut self, base: Nullable<IriString>) {
        self.base = base;
    }

    /// Sets the default base direction.
    pub(crate) fn set_default_base_direction(&mut self, dir: Option<definition::Direction>) {
        self.default_base_direction = dir;
    }

    /// Sets the default language.
    pub(crate) fn set_default_language(&mut self, lang: Option<String>) {
        self.default_language = lang;
    }

    /// Returns a raw term definition.
    ///
    /// This distinguishes absence and explicit `null`.
    pub(crate) fn raw_term_definition(&self, term: &'_ str) -> Option<Nullable<&Definition>> {
        self.term_definitions.get(term).map(Nullable::as_ref)
    }

    /// Returns a flattened term definition.
    ///
    /// This returns `None` for both absent term and term set to explicit `null`.
    pub(crate) fn term_definition(&self, term: &'_ str) -> Option<&Definition> {
        self.term_definitions
            .get(term)
            .and_then(|v| v.as_ref().into())
    }

    /// Removes the given term definition.
    ///
    /// This does nothing if the given term is not in the context.
    pub(crate) fn remove_term_definition(&mut self, term: &str) -> Option<Nullable<Definition>> {
        self.term_definitions.remove(term)
    }

    /// Runs create term definition algorithm.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#create-term-definition>.
    pub(crate) async fn create_term_definition<L: LoadRemoteDocument>(
        &mut self,
        processor: &Processor<L>,
        local_context: &JsonMap<String, Value>,
        term: &str,
        defined: &mut HashMap<String, bool>,
    ) -> Result<()> {
        create_term_definition(
            processor,
            self,
            local_context,
            term,
            defined,
            CreateTermDefOptionalParams::new(),
        )
        .await
    }

    /// Returns the vocabulary mapping.
    pub(crate) fn vocab(&self) -> Nullable<&str> {
        self.vocab.as_ref().map(AsRef::as_ref)
    }

    /// Sets the vocabulary mapping.
    pub(crate) fn set_vocab(&mut self, vocab: impl Into<Nullable<String>>) {
        self.vocab = vocab.into();
    }

    /// Checks whether the context has the previous context.
    pub(crate) fn has_previous_context(&self) -> bool {
        self.previous_context.is_some()
    }

    /// Checks whether the context has any protected term definition.
    pub(crate) fn has_protected_term_definition(&self) -> bool {
        self.term_definitions
            .values()
            .filter_map(|nullable| Into::<Option<_>>::into(nullable.as_ref()))
            .any(Definition::is_protected)
    }

    /// Runs context processing algorithm and returns a new context.
    ///
    /// This receives a value associated to `@context` key.
    /// If you want to pass a JSON value which contains `@context` entry, use
    /// `Context::join_context_document` instead.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#context-processing-algorithm>.
    pub async fn join_context_value<L: LoadRemoteDocument>(
        &self,
        processor: &Processor<L>,
        local_context: &Value,
        override_protected: bool,
    ) -> Result<Self> {
        merge::join_value(
            processor,
            self,
            local_context,
            MergeOptionalParams::new().override_protected(override_protected),
        )
        .await
    }

    /// Runs context processing algorithm and returns a new context.
    ///
    /// This receives a JSON value which contains `@context` entry.
    /// If you want to pass a value associated to `@context` key, use `Context::join_context_value`
    /// instead.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#context-processing-algorithm>.
    pub async fn join_context_document<L: LoadRemoteDocument>(
        &self,
        processor: &Processor<L>,
        context_doc: &Value,
        override_protected: bool,
    ) -> Result<Self> {
        if let Some(local_context) = context_doc.get("@context") {
            self.join_context_value(processor, local_context, override_protected)
                .await
        } else {
            Ok(self.clone())
        }
    }
}
