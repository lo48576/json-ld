//! "Create term definition" algorithm.

use std::{borrow::Cow, collections::HashMap, future::Future, pin::Pin};

use anyhow::anyhow;
use serde_json::{Map as JsonMap, Value};

use crate::{
    context::{definition::DefinitionBuilder, Context, ValueWithBase},
    error::{ErrorCode, Result},
    expand::iri::ExpandIriOptions,
    iri::is_absolute_iri_ref,
    json::single_entry_map,
    processor::{Processor, ProcessorOptions},
    remote::LoadRemoteDocument,
    syntax::has_form_of_keyword,
};

use self::{non_reverse::run_for_non_reverse, reverse::run_for_reverse};

mod non_reverse;
mod reverse;

/// Optional parameters (arguments) for create term definition algorithm.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OptionalParams {
    /// Protected.
    protected: bool,
    /// Override protected.
    override_protected: bool,
    /// Propagate.
    propagate: bool,
}

impl OptionalParams {
    /// Sets the `protected` option if available.
    pub(crate) fn protected_opt(self, protected: Option<bool>) -> Self {
        Self {
            protected: protected.unwrap_or(self.protected),
            ..self
        }
    }

    /// Sets the `propagate` option if available.
    pub(crate) fn propagate(self, propagate: bool) -> Self {
        Self { propagate, ..self }
    }
}

impl Default for OptionalParams {
    fn default() -> Self {
        Self {
            protected: false,
            override_protected: false,
            propagate: true,
        }
    }
}

impl OptionalParams {
    /// Creates a new `CreateTermDefOptions`.
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

/// Runs create term definition algorithm.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#create-term-definition>
pub(crate) fn create_term_definition<'a, L: LoadRemoteDocument>(
    processor: &'a Processor<L>,
    active_context: &'a mut Context,
    local_context: ValueWithBase<'a, &'a JsonMap<String, Value>>,
    term: &'a str,
    defined: &'a mut HashMap<String, bool>,
    optional: OptionalParams,
) -> Pin<Box<dyn Future<Output = Result<()>> + 'a + Send>> {
    Box::pin(async move {
        create_term_definition_impl(
            processor,
            active_context,
            local_context,
            term,
            defined,
            optional,
        )
        .await
    })
}

/// Runs create term definition algorithm.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#create-term-definition>
async fn create_term_definition_impl<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &mut Context,
    local_context: ValueWithBase<'_, &JsonMap<String, Value>>,
    term: &str,
    defined: &mut HashMap<String, bool>,
    optional: OptionalParams,
) -> Result<()> {
    use std::collections::hash_map::Entry;

    // Step 1, 2
    match defined.entry(term.into()) {
        Entry::Occupied(entry) => {
            // Step 1
            if *entry.get() {
                // Term definition for `term` has already been created.
                return Ok(());
            } else {
                return Err(ErrorCode::CyclicIriMapping.and_source(anyhow!("term = {:?}", term)));
            }
        }
        Entry::Vacant(entry) => {
            // Step 2
            entry.insert(false);
        }
    }
    debug_assert!(
        defined.contains_key(term),
        "`defined` should have an entry for `term` (= {:?})",
        term
    );
    // Step 3
    let value = local_context.value().get(term).unwrap_or_else(|| {
        panic!(
            "Should never fail: the given `term` should have been chosen from `local_context`
             keys: term={:?}, local_context={:?}",
            term, local_context
        )
    });
    // Step 4
    if term == "@type" {
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::KeywordRedefinition.and_source(anyhow!(
                "`term` = \"@type\" and processing mode is `json-ld-1.0`"
            )));
        }
        let map = match value {
            Value::Object(map) => map,
            v => {
                return Err(ErrorCode::KeywordRedefinition.and_source(anyhow!(
                    "Expected an object for term `@type`, but got {:?}",
                    v
                )))
            }
        };
        // > At this point, value *MUST* be a map with only the entry `@container` and value
        // > `@set` and optional entry `@protected`.
        if map.get("@container").and_then(|v| v.as_str()) != Some("@set") {
            return Err(ErrorCode::KeywordRedefinition.and_source(anyhow!(
                "Expected the value `@set` for `@container` entry for term `@type`, but got {:?}",
                map.get("@container")
            )));
        }
        if let Some((k, v)) = map
            .iter()
            .find(|(k, _)| *k != "@container" && *k != "@protected")
        {
            return Err(ErrorCode::KeywordRedefinition.and_source(anyhow!(
                "Unexpected entry for term `@type`: key={:?}, value={:?}",
                k,
                v
            )));
        }
    }
    // Step 5
    if processor.is_keyword(term) {
        // Keywords cannot be overridden.
        return Err(ErrorCode::KeywordRedefinition.and_source(anyhow!("term = {:?}", term)));
    }
    if has_form_of_keyword(term) {
        // TODO: Generate a warning.
        return Ok(());
    }
    // Step 6
    // If the (previous) definition is explicit `null`, treat it as absent.
    let previous_definition: Option<_> = active_context
        .remove_term_definition(term)
        .and_then(Into::into);
    // Step 7-9
    let (value, simple_term) = match value {
        // Step 7
        Value::Null => (Cow::Owned(single_entry_map("@id", Value::Null)), false),
        // Step 8
        Value::String(s) => (Cow::Owned(single_entry_map("@id", s.clone())), true),
        // Step 9
        Value::Object(v) => (Cow::Borrowed(v), false),
        // Step 9
        v => return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!("value = {:?}", v))),
    };
    // Step 10
    let mut definition = DefinitionBuilder::new();
    // Step 11, 12
    process_protected(processor.options(), optional, &value, &mut definition)?;
    // Step 13
    process_type(
        processor,
        active_context,
        local_context,
        defined,
        &value,
        &mut definition,
    )
    .await?;
    // Step 14-
    if let Some(reverse) = value.get("@reverse") {
        // Step 14
        run_for_reverse(
            processor,
            active_context,
            local_context,
            term,
            defined,
            &value,
            reverse,
            definition,
        )
        .await
    } else {
        // Step 15-
        run_for_non_reverse(
            processor,
            active_context,
            local_context,
            term,
            defined,
            optional,
            &value,
            definition,
            previous_definition,
            simple_term,
        )
        .await
    }
}

/// Processes the "protected" flag.
fn process_protected(
    processor: &ProcessorOptions,
    optional: OptionalParams,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
    // Step 11, 12
    match value.get("@protected") {
        // Step 11
        Some(Value::Bool(true)) => {
            if processor.is_processing_mode_1_0() {
                return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                    "`@protected` is `true` but processing mode is `json-ld-1.0`"
                )));
            }
            definition.set_protected(true);
        }
        // Step 11
        Some(Value::Bool(false)) => {}
        // Step 11
        Some(v) => {
            return Err(ErrorCode::InvalidProtectedValue.and_source(anyhow!(
                "Expected boolean or `null` as `@protected`, but got {:?}",
                v,
            )))
        }
        // Step 12
        None if optional.protected => {
            definition.set_protected(true);
        }
        _ => {}
    }

    Ok(())
}

/// Processes the type mapping.
async fn process_type<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &mut Context,
    local_context: ValueWithBase<'_, &JsonMap<String, Value>>,
    defined: &mut HashMap<String, bool>,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
    // Step 13
    match value.get("@type") {
        // Step 13.1
        Some(Value::String(ty)) => {
            // Step 13.2, 13.4
            let ty = ExpandIriOptions::mutable(active_context, local_context, defined)
                .vocab(true)
                .expand_str(processor, ty)
                .await?
                .ok_or_else(|| {
                    ErrorCode::InvalidTypeMapping
                        .and_source(anyhow!("@type ({:?}) is expanded to `null`", ty))
                })?;
            // Step 13.3
            if (ty == "@json" || ty == "@none") && processor.is_processing_mode_1_0() {
                return Err(ErrorCode::InvalidTypeMapping.and_source(anyhow!(
                    "@type = {:?} while processing mode is JSON-LD-1.0",
                    ty
                )));
            }
            // Step 13.4, 13.5
            if ty == "@id" || ty == "@vocab" || is_absolute_iri_ref(&ty) {
                definition.set_ty(ty);
            } else {
                return Err(
                    ErrorCode::InvalidTypeMapping.and_source(anyhow!("expanded type = {:?}", ty))
                );
            }
        }
        None => {}
        // Step 13.1
        v => return Err(ErrorCode::InvalidTypeMapping.and_source(anyhow!("@type = {:?}", v))),
    }

    Ok(())
}
