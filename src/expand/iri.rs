//! IRI expansion.
//!
//! See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#iri-expansion>.

use std::{borrow::Cow, collections::HashMap};

use iri_string::types::IriReferenceStr;
use serde_json::{Map as JsonMap, Value};

use crate::{
    context::{Context, Definition, ValueWithBase},
    error::{ErrorCode, Result},
    iri::{is_absolute_iri_ref, to_prefix_and_suffix},
    json::Nullable,
    processor::Processor,
    remote::LoadRemoteDocument,
    syntax::has_form_of_keyword,
};

/// Context for IRI expansion.
#[derive(Debug)]
enum ExpandIriContext<'a> {
    /// Immutable context.
    Constant {
        /// Active context.
        active_context: &'a Context,
    },
    /// Mutable context.
    Mutable {
        /// Active context.
        active_context: &'a mut Context,
        /// Local (currently loading) context.
        local_context: ValueWithBase<'a, &'a JsonMap<String, Value>>,
        /// Terms defined and being defined.
        defined: &'a mut HashMap<String, bool>,
    },
}

impl<'a> ExpandIriContext<'a> {
    /// Creates a new `ExpandIriContext` with the given immutable context.
    fn constant(active_context: &'a Context) -> Self {
        ExpandIriContext::Constant { active_context }
    }

    /// Creates a new `ExpandIriContext` with the given mutable context.
    fn mutable(
        active_context: &'a mut Context,
        local_context: ValueWithBase<'a, &'a JsonMap<String, Value>>,
        defined: &'a mut HashMap<String, bool>,
    ) -> Self {
        ExpandIriContext::Mutable {
            active_context,
            local_context,
            defined,
        }
    }
}

/// Options for IRI expansion algorithm.
#[derive(Debug)]
pub(crate) struct ExpandIriOptions<'a> {
    /// Context.
    context: ExpandIriContext<'a>,
    /// Vocab.
    vocab: bool,
    /// Document relative.
    document_relative: bool,
}

impl<'a> ExpandIriOptions<'a> {
    /// Creates a new `ExpandIriOptions` with the given immutable context.
    #[allow(dead_code)]
    pub(crate) fn constant(active_context: &'a Context) -> Self {
        Self {
            context: ExpandIriContext::constant(active_context),
            vocab: false,
            document_relative: false,
        }
    }

    /// Creates a new `ExpandIriOptions` with the given mutable context.
    #[allow(dead_code)]
    pub(crate) fn mutable(
        active_context: &'a mut Context,
        local_context: ValueWithBase<'a, &'a JsonMap<String, Value>>,
        defined: &'a mut HashMap<String, bool>,
    ) -> Self {
        Self {
            context: ExpandIriContext::mutable(active_context, local_context, defined),
            document_relative: false,
            vocab: false,
        }
    }

    /// Sets "document relative" flag.
    #[allow(dead_code)]
    pub(crate) fn document_relative(self, document_relative: bool) -> Self {
        Self {
            document_relative,
            ..self
        }
    }

    /// Sets "vocab" flag.
    #[allow(dead_code)]
    pub(crate) fn vocab(self, vocab: bool) -> Self {
        Self { vocab, ..self }
    }

    /// Returns the active context.
    fn active_context(&self) -> &Context {
        match &self.context {
            ExpandIriContext::Constant { active_context } => active_context,
            ExpandIriContext::Mutable { active_context, .. } => active_context,
        }
    }

    /// Returns the raw term definition if exists, or `self`.
    fn into_raw_term_definition(
        self,
        term: &str,
    ) -> std::result::Result<Nullable<&'a Definition>, ExpandIriOptions<'a>> {
        let Self {
            context,
            vocab,
            document_relative,
        } = self;
        match context {
            ExpandIriContext::Constant { active_context } => {
                if let Some(def) = active_context.raw_term_definition(term) {
                    Ok(def)
                } else {
                    Err(Self {
                        context: ExpandIriContext::Constant { active_context },
                        vocab,
                        document_relative,
                    })
                }
            }
            ExpandIriContext::Mutable {
                active_context,
                local_context,
                defined,
            } => {
                // NOTE: Using `expect()` after `is_some()` is necessary, because the code below
                // does not compile with rust 1.38.0.
                //
                // ```
                // match active_context.raw_term_definition(term) {
                //     Some(def) => Ok(def),
                //     None => Err(/* expr consuming `active_context` */),
                // }
                // ```
                if active_context.raw_term_definition(term).is_some() {
                    let def = active_context
                        .raw_term_definition(term)
                        .expect("Should never fail: already checked by `is_some()`");
                    Ok(def)
                } else {
                    Err(Self {
                        context: ExpandIriContext::Mutable {
                            active_context,
                            local_context,
                            defined,
                        },
                        vocab,
                        document_relative,
                    })
                }
            }
        }
    }

    /// Runs "create term definition" algorithm if necessary.
    async fn create_term_definition<L: LoadRemoteDocument>(
        &mut self,
        processor: &Processor<L>,
        value: &str,
    ) -> Result<()> {
        if let ExpandIriContext::Mutable {
            active_context,
            local_context,
            defined,
        } = &mut self.context
        {
            if local_context.value().contains_key(value) && defined.get(value) != Some(&true) {
                active_context
                    .create_term_definition(processor, *local_context, value, defined)
                    .await?;
            }
        }

        Ok(())
    }

    /// Runs IRI expansion algorithm for string value.
    ///
    /// This may return one of the below:
    ///
    /// * `Ok(Some(absolute_iri_reference))`
    /// * `Ok(Some(blank_node_identifier))`
    /// * `Ok(None)`
    ///     + This means the value is successfully expanded to `null`.
    /// * `Err(_)`
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#iri-expansion>.
    pub(crate) async fn expand_str<L: LoadRemoteDocument>(
        self,
        processor: &Processor<L>,
        value: &'a str,
    ) -> Result<Option<Cow<'a, str>>> {
        expand_str(self, processor, value).await
    }

    /// Runs IRI expansion algorithm for string value and returns JSON value.
    ///
    /// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#iri-expansion>.
    #[allow(dead_code)]
    pub(crate) async fn expand_to_json<L: LoadRemoteDocument>(
        self,
        processor: &Processor<L>,
        value: &str,
    ) -> Result<Value> {
        Ok(self
            .expand_str(processor, value)
            .await?
            .map_or(Value::Null, |s| Value::String(s.into())))
    }
}

/// Runs IRI expansion algorithm for string value.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#iri-expansion>.
async fn expand_str<'a, L: LoadRemoteDocument>(
    mut options: ExpandIriOptions<'a>,
    processor: &Processor<L>,
    value: &'a str,
) -> Result<Option<Cow<'a, str>>> {
    // Step 1
    if processor.is_keyword(value) {
        return Ok(Some(Cow::Borrowed(value)));
    }
    // Step 2
    if has_form_of_keyword(value) {
        // TODO: Generate a warning.
        return Ok(None);
    }
    // Step 3
    options.create_term_definition(processor, value).await?;
    // Step 4
    if let Some(keyword) = options
        .active_context()
        .term_definition(value)
        .map(|def| def.iri())
        .filter(|iri| processor.is_keyword(iri))
    {
        // Return a keyword.
        return Ok(Some(Cow::Owned(keyword.to_owned())));
    }
    // Step 5
    if options.vocab {
        // NOTE: If the term is mapped to `null`, it indicates not only that the term is not mapped
        // to any term, but also that the term should be decoupled from `@vocab`.
        // See W3C test t0032.
        options = match options.into_raw_term_definition(value) {
            Ok(def) => match def {
                Nullable::Null => return Ok(None),
                Nullable::Value(def) => return Ok(Some(Cow::Borrowed(def.iri()))),
            },
            Err(options) => options,
        };
    }
    // Step 6
    if let Some((prefix, suffix)) = to_prefix_and_suffix(value) {
        debug_assert!(!prefix.is_empty());
        // Step 6.2: `value` is either an IRI, a compact IRI, or a blank node identifier.
        if prefix == "_" || suffix.starts_with("//") {
            // `value` is already an IRI or a blank node identifier.
            return Ok(Some(Cow::Borrowed(value)));
        }
        // Step 6.3
        options.create_term_definition(processor, prefix).await?;
        // Step 6.4
        // NOTE: Treat prefix as not defined if it is mapped to `null`.
        if let Some(prefix_def) = options
            .active_context()
            .term_definition(prefix)
            .filter(|def| def.is_prefix())
        {
            return Ok(Some(Cow::Owned(format!("{}{}", prefix_def.iri(), suffix))));
        }
        // Step 6.5
        if is_absolute_iri_ref(value) {
            // `value` is already an IRI.
            return Ok(Some(Cow::Borrowed(value)));
        }
    }
    // Step 7
    if options.vocab {
        if let Nullable::Value(vocab) = options.active_context().vocab() {
            return Ok(Some(Cow::Owned(format!("{}{}", vocab, value))));
        }
    }
    // Step 8
    if options.document_relative {
        // NOTE: This is base IRI from the active context, not the raw document IRI.
        // See <https://github.com/w3c/json-ld-api/issues/180#issuecomment-547177451>.
        let base = match options.active_context().base() {
            Nullable::Value(base) => base,
            Nullable::Null => {
                // Not sure what to do when the base is explicitly nullified.
                return Err(ErrorCode::Uncategorized.and_source(anyhow::anyhow!(
                    "`document_relative` is true but base IRI from the active context is `null`",
                )));
            }
        };
        let value: &IriReferenceStr = IriReferenceStr::new(value).map_err(|e| {
            ErrorCode::Uncategorized.and_source(anyhow::anyhow!(
                "Attempt to resolve {:?} as IRI, but it is not actually valid IRI: {}",
                value,
                e
            ))
        })?;
        return Ok(Some(Cow::Owned(
            value.resolve_against(base.to_absolute()).into(),
        )));
    }

    // Step 9
    Ok(Some(Cow::Borrowed(value)))
}
