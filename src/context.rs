//! JSON-LD context.
//!
//! See <https://www.w3.org/TR/2019/WD-json-ld11-20191018/#the-context>.

use std::collections::HashMap;

use iri_string::types::{IriStr, IriString};
use serde_json::{Map as JsonMap, Value};

use crate::{error::Result, json::Nullable, processor::ProcessorOptions};

use self::create_term_def::{create_term_definition, OptionalParams};
pub(crate) use self::definition::Definition;

mod create_term_def;
mod definition;

/// JSON-LD context.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-20191018/#the-context>.
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    /// Term definitions.
    term_definitions: HashMap<String, Nullable<Definition>>,
    /// Base IRI.
    base: Option<Nullable<IriString>>,
    /// Vocabulary mapping.
    vocab: Option<String>,
}

impl Context {
    /// Returns the base IRI.
    pub(crate) fn base(&self) -> Option<Nullable<&IriStr>> {
        self.base.as_ref().map(|v| v.as_ref().map(AsRef::as_ref))
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
    pub(crate) fn create_term_definition(
        &mut self,
        processor: &ProcessorOptions,
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
            OptionalParams::new(),
        )
    }

    /// Returns the vocabulary mapping.
    pub(crate) fn vocab(&self) -> Option<&str> {
        self.vocab.as_ref().map(AsRef::as_ref)
    }
}
