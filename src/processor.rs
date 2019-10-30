//! JSON-LD processor.
//!
//! See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#the-jsonldprocessor-interface>.

use std::borrow::Cow;

use iri_string::types::{IriStr, IriString};

use crate::{context::Context, json::Nullable};

/// JSON-LD processor options.
///
/// See <https://www.w3.org/TR/2014/REC-json-ld-api-20140116/#the-jsonldoptions-type>.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorOptions {
    /// Base IRI (or document IRI).
    document_iri: IriString,
}

impl ProcessorOptions {
    /// Returns the base IRI set by the processor.
    pub(crate) fn document_iri(&self) -> &IriStr {
        self.document_iri.as_ref()
    }

    /// Checks if the given string is a keyword.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-20191018/#syntax-tokens-and-keywords>.
    pub(crate) fn is_keyword(&self, _: &str) -> bool {
        unimplemented!()
    }

    /// Returns the base IRI.
    ///
    /// Note that the base can be empty (null) when `{ "@context": { "@base": null } }` is
    /// specified.
    pub(crate) fn base<'a>(&'a self, context: &'a Context) -> Option<Cow<'a, IriStr>> {
        match context.base() {
            Some(Nullable::Value(context_base)) => match context_base.to_iri() {
                Ok(iri) => Some(Cow::Borrowed(iri)),
                Err(_) => Some(Cow::Owned(
                    context_base.resolve_against(self.document_iri().to_absolute()),
                )),
            },
            Some(Nullable::Null) => None,
            None => Some(Cow::Borrowed(self.document_iri())),
        }
    }
}
