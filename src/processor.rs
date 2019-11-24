//! JSON-LD processor.
//!
//! See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#the-jsonldprocessor-interface>.

use std::borrow::Cow;

use iri_string::types::{IriStr, IriString};

use crate::{context::Context, json::Nullable, remote::LoadRemoteDocument};

/// JSON-LD processor options.
///
/// See <https://www.w3.org/TR/2014/REC-json-ld-api-20140116/#the-jsonldoptions-type>.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorOptions {
    /// Base IRI (or document IRI).
    document_iri: IriString,
}

impl ProcessorOptions {
    /// Creates a new `ProcessorOptions` from the given base IRI (or document IRI).
    pub fn with_base(document_iri: impl Into<IriString>) -> Self {
        Self {
            document_iri: document_iri.into(),
        }
    }

    /// Returns the base IRI set by the processor.
    pub(crate) fn document_iri(&self) -> &IriStr {
        self.document_iri.as_ref()
    }

    /// Checks if the processing mode is `json-ld-1.0`.
    pub(crate) fn is_processing_mode_1_0(&self) -> bool {
        // Currently unsupported.
        false
    }

    /// Checks if the given string is a keyword.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-20191112/#syntax-tokens-and-keywords>.
    pub(crate) fn is_keyword(&self, s: &str) -> bool {
        /// Keywords in JSON-LD 1.1.
        ///
        /// See <https://www.w3.org/TR/2019/WD-json-ld11-20191112/#syntax-tokens-and-keywords>.
        const KEYWORDS_1_1: &[&str] = &[
            "@base",
            "@container",
            "@context",
            "@direction",
            "@graph",
            "@id",
            "@import",
            "@included",
            "@index",
            "@json",
            "@language",
            "@list",
            "@nest",
            "@none",
            "@prefix",
            "@propagate",
            "@protected",
            "@reverse",
            "@set",
            "@type",
            "@value",
            "@version",
            "@vocab",
        ];
        KEYWORDS_1_1.contains(&s)
    }

    /// Returns the base IRI.
    ///
    /// Note that the base can be empty (null) when `{ "@context": { "@base": null } }` is
    /// specified.
    pub(crate) fn base<'a>(&'a self, context: &'a Context) -> Option<Cow<'a, IriStr>> {
        match context.base() {
            Nullable::Value(context_base) => match context_base.to_iri() {
                Ok(iri) => Some(Cow::Borrowed(iri)),
                Err(_) => Some(Cow::Owned(
                    context_base.resolve_against(self.document_iri().to_absolute()),
                )),
            },
            Nullable::Null => None,
        }
    }

    /// Returns the limit of number of remote contexts.
    ///
    /// If `Some(n)` is returned, `n` remote contexts is allowed, and one more remote context will
    /// be rejected.
    /// `None` means there are no limits.
    pub(crate) fn allowed_max_remote_context(&self) -> Option<usize> {
        unimplemented!()
    }

    /// Creates a processor from the option and the given loader.
    pub fn build<L: LoadRemoteDocument>(self, loader: L) -> Processor<L> {
        Processor {
            options: self,
            loader,
        }
    }
}

/// JSON-LD processor.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#the-jsonldprocessor-interface>
/// and <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#the-jsonldoptions-type>.
pub struct Processor<L> {
    /// Processor options (except a loader).
    options: ProcessorOptions,
    /// Remote context loader.
    loader: L,
}

impl<L: LoadRemoteDocument> Processor<L> {
    /// Returns processor options.
    pub fn options(&self) -> &ProcessorOptions {
        &self.options
    }

    /// Returns processor options.
    pub fn loader(&self) -> &L {
        &self.loader
    }
}

impl<L: LoadRemoteDocument> Processor<L> {
    /// Checks if the processing mode is `json-ld-1.0`.
    pub(crate) fn is_processing_mode_1_0(&self) -> bool {
        self.options().is_processing_mode_1_0()
    }

    /// Checks if the given string is a keyword.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-20191112/#syntax-tokens-and-keywords>.
    pub(crate) fn is_keyword(&self, s: &str) -> bool {
        self.options().is_keyword(s)
    }

    /// Returns the base IRI.
    ///
    /// Note that the base can be empty (null) when `{ "@context": { "@base": null } }` is
    /// specified.
    pub(crate) fn base<'a>(&'a self, context: &'a Context) -> Option<Cow<'a, IriStr>> {
        self.options().base(context)
    }

    /// Checks if the number of context exceeds the processor limit.
    pub(crate) fn is_remote_context_limit_exceeded(&self, num_ctx: usize) -> bool {
        match self.options().allowed_max_remote_context() {
            Some(max_allowed) => num_ctx > max_allowed,
            None => false,
        }
    }
}
