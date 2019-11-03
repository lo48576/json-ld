//! "Create term definition" algorithm.

use std::{borrow::Cow, collections::HashMap, convert::TryFrom};

use anyhow::anyhow;
use serde_json::{Map as JsonMap, Value};

use crate::{
    context::{
        definition::{Container, ContainerItem, Definition, DefinitionBuilder, Direction},
        Context,
    },
    error::{ErrorCode, Result},
    expand::iri::ExpandIriOptions,
    iri::{
        is_absolute_iri, is_absolute_or_blank_node_ident, is_compact_iri, is_gen_delims_byte,
        to_prefix_and_suffix,
    },
    json::{single_entry_map, Nullable},
    processor::ProcessorOptions,
};

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
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#create-term-definition>
pub(crate) fn create_term_definition(
    processor: &ProcessorOptions,
    active_context: &mut Context,
    local_context: &JsonMap<String, Value>,
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
    let value = local_context.get(term).unwrap_or_else(|| {
        panic!(
            "Should never fail: the given `term` should have been chosen from `local_context`
             keys: term={:?}, local_context={:?}",
            term, local_context
        )
    });
    // Step 4
    if term == "@type" && processor.is_processing_mode_1_0() {
        return Err(ErrorCode::KeywordRedefinition.and_source(anyhow!(
            "`term` = \"@type\" and processing mode is `json-ld-1.0`"
        )));
    }
    match value {
        Value::Object(map) => {
            if map.get("@container").and_then(|v| v.as_str()) != Some("@set") {
                // TODO: What to do if this "MUST" condition is not met?
                //
                // > At this point, value *MUST* be a map with only the entry `@container` and value
                // > `@set` and optional entry `@protected`.
                return Err(ErrorCode::Uncategorized.and_source(anyhow!(
                    "Expected the value `@set` for `@container` entry, but got {:?}",
                    map.get("@container")
                )));
            }
            if let Some((k, v)) = map
                .iter()
                .find(|(k, _)| *k != "@container" && *k != "@protected")
            {
                return Err(ErrorCode::KeywordRedefinition.and_source(anyhow!(
                    "Unexpected entry: key={:?}, value={:?}",
                    k,
                    v
                )));
            }
        }
        Value::Null => {}
        _ => {
            return Err(ErrorCode::KeywordRedefinition
                .and_source(anyhow!("Unexpected type: value={:?}", value)))
        }
    }
    // Step 5
    if processor.is_keyword(term) {
        // Keywords cannot be overridden.
        return Err(ErrorCode::KeywordRedefinition.and_source(anyhow!("term = {:?}", term)));
    }
    if term.starts_with('@') {
        // TODO: Generate a warning.
        // TODO: How to "abort processing" here? No error code is explicitly specified in the spec.
        // See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#algorithm-0>.
        return Err(ErrorCode::Uncategorized
            .and_source(anyhow!("term has the form of a keyword: term = {:?}", term)));
    }
    // Step 6
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    // TODO: How to treat `null`?
    let previous_definition = active_context.remove_term_definition(term);
    // Step 7-9
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    let (value, simple_term) = match value {
        // Step 7
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        Value::Null => (Cow::Owned(single_entry_map("@id", Value::Null)), false),
        // Step 8
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        Value::String(s) => (Cow::Owned(single_entry_map("@id", s.clone())), true),
        // Step 9
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        Value::Object(v) => (Cow::Borrowed(v), false),
        v => return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!("value = {:?}", v))),
    };
    // Step 10
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    let mut definition = DefinitionBuilder::new();
    // Step 11, 12
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    match value.get("@protected") {
        // Step 11
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        Some(Value::Bool(true)) => {
            if processor.is_processing_mode_1_0() {
                return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                    "`@protected` is `true` but processing mode is `json-ld-1.0`"
                )));
            }
            definition.set_protected(true);
        }
        // Step 12
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        None if optional.protected => {
            definition.set_protected(true);
        }
        _ => {}
    }
    // Step 13
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    match value.get("@type") {
        // Step 13.1
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        Some(Value::String(ty)) => {
            // Step 13.2, 13.4
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            let ty = ExpandIriOptions::mutable(active_context, local_context, defined)
                .vocab(true)
                .expand_str(processor, ty)?
                .ok_or_else(|| {
                    ErrorCode::InvalidTypeMapping
                        .and_source(anyhow!("@type ({:?}) is expanded to `null`", ty))
                })?;
            // Step 13.3
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            if (ty == "@json" || ty == "@none") && processor.is_processing_mode_1_0() {
                return Err(ErrorCode::InvalidTypeMapping.and_source(anyhow!(
                    "@type = {:?} while processing mode is JSON-LD-1.0",
                    ty
                )));
            }
            // Step 13.4, 13.5
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            if ty == "@id" || ty == "@vocab" || is_absolute_iri(&ty) {
                definition.set_ty(ty);
            } else {
                return Err(
                    ErrorCode::InvalidTypeMapping.and_source(anyhow!("expanded type = {:?}", ty))
                );
            }
        }
        None => {}
        // Step 13.1
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        v => return Err(ErrorCode::InvalidTypeMapping.and_source(anyhow!("@type = {:?}", v))),
    }
    // Step 14
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(reverse) = value.get("@reverse") {
        run_for_reverse(
            processor,
            active_context,
            local_context,
            term,
            defined,
            optional,
            &value,
            reverse,
            definition,
        )
    } else {
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
    }
}

/// Runs rest of the create term definition algorithm for the case `@reverse` exists.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#create-term-definition>
// Step 14
// NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
// as WD-json-ld11-api-20191018 has ambiguity.
#[allow(clippy::too_many_arguments)] // TODO: FIXME
fn run_for_reverse(
    processor: &ProcessorOptions,
    active_context: &mut Context,
    local_context: &JsonMap<String, Value>,
    term: &str,
    defined: &mut HashMap<String, bool>,
    _optional: OptionalParams,
    value: &JsonMap<String, Value>,
    reverse: &Value,
    mut definition: DefinitionBuilder,
) -> Result<()> {
    // Step 14.1
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if value.contains_key("@id") || value.contains_key("@nest") {
        return Err(
            ErrorCode::InvalidReverseProperty.and_source(anyhow!("Found `@id` or `@nest` entries"))
        );
    }
    // Step 14.2
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    let reverse = match reverse {
        Value::String(s) => s,
        v => {
            return Err(ErrorCode::InvalidIriMapping
                .and_source(anyhow!("Expected string as @reverse but got {:?}", v)))
        }
    };
    // Step 14.3
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if reverse.starts_with('@') {
        // FIXME: Generate a warning.
        // TODO: How to "abort processing" here? No error code is explicitly specified in the spec.
        return Err(ErrorCode::Uncategorized
            .and_source(anyhow!("@reverse value ({:?}) starts with `@`", reverse)));
    }
    // Step 14.4
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    let reverse = ExpandIriOptions::mutable(active_context, local_context, defined)
        .vocab(true)
        .expand_str(processor, reverse)?
        .ok_or_else(|| {
            ErrorCode::InvalidIriMapping
                .and_source(anyhow!("@reverse ({:?}) is expanded to `null`", reverse))
        })?;
    if is_absolute_or_blank_node_ident(&reverse) {
        definition.set_iri(reverse);
    } else {
        return Err(ErrorCode::InvalidIriMapping.and_source(anyhow!(
            "Expanded @reverse value ({:?}) is neither an IRI nor blank node identifier",
            reverse
        )));
    }
    // Step 14.5
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(container) = value.get("@container") {
        let container = Nullable::<Container>::try_from(container)
            .map_err(|e| ErrorCode::InvalidContainerMapping.and_source(e))?;
        match container {
            Nullable::Null
            | Nullable::Value(Container::Single(ContainerItem::Set))
            | Nullable::Value(Container::Single(ContainerItem::Index)) => {
                definition.set_container(container);
            }
            v => {
                return Err(
                    ErrorCode::InvalidReverseProperty.and_source(anyhow!("`@container` = {:?}", v))
                )
            }
        }
    }
    // Step 14.6
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    definition.set_reverse(true);
    // Step 14.7
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    let definition = definition.build();
    active_context
        .term_definitions
        .insert(term.to_owned(), Nullable::Value(definition));
    *defined
        .get_mut(term)
        .expect("Should never fail: inserted before") = true;

    Ok(())
}

/// Runs rest of the create term definition algorithm for the case `@reverse` exists.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#create-term-definition>
// Step 15 and after.
// NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
// as WD-json-ld11-api-20191018 has ambiguity.
#[allow(clippy::too_many_arguments)] // TODO: FIXME
fn run_for_non_reverse(
    processor: &ProcessorOptions,
    active_context: &mut Context,
    local_context: &JsonMap<String, Value>,
    term: &str,
    defined: &mut HashMap<String, bool>,
    optional: OptionalParams,
    value: &JsonMap<String, Value>,
    mut definition: DefinitionBuilder,
    previous_definition: Option<Nullable<Definition>>,
    simple_term: bool,
) -> Result<()> {
    // Step 15
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    definition.set_reverse(false);
    // Step 16
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(id) = value.get("@id") {
        match id {
            // Step 16
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            Value::String(id) if id == term => {}
            // Step 16.1
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            Value::Null => {
                unimplemented!("TODO: Do something?");
            }
            // Step 16.3-
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            Value::String(id) => {
                // Step 16.3
                // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                // as WD-json-ld11-api-20191018 has ambiguity.
                if !processor.is_keyword(id) && id.starts_with('@') {
                    // TODO: Generate warning.
                    return Err(ErrorCode::Uncategorized.and_source(anyhow!(
                        "@id value {:?} is not a keyword but starts with `@`",
                        id
                    )));
                }
                // Step 16.4
                // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                // as WD-json-ld11-api-20191018 has ambiguity.
                let id = ExpandIriOptions::mutable(active_context, local_context, defined)
                    .vocab(true)
                    .expand_str(processor, id)?
                    .ok_or_else(|| {
                        ErrorCode::InvalidIriMapping
                            .and_source(anyhow!("@id ({:?}) is expanded to `null`", id))
                    })?;
                if !processor.is_keyword(&id) && !is_absolute_or_blank_node_ident(&id) {
                    return Err(ErrorCode::InvalidIriMapping.and_source(anyhow!(
                        "@id ({:?}) should be a keyword, \
                         an IRI (which is absolute), or a blank node identifier",
                        id
                    )));
                } else if id == "@context" {
                    return Err(ErrorCode::InvalidKeywordAlias
                        .and_source(anyhow!("Invalid alias to `@context`")));
                }
                // Step 16.5
                // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                // as WD-json-ld11-api-20191018 has ambiguity.
                if (!term.is_empty() && term[..(term.len() - 1)].contains(':'))
                    || term.contains('/')
                {
                    return Err(
                        ErrorCode::InvalidIriMapping.and_source(anyhow!("term = {:?}", term))
                    );
                }
                // Step 16.6
                // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                // as WD-json-ld11-api-20191018 has ambiguity.
                if !term.contains(':')
                    && !term.contains('/')
                    && simple_term
                    && is_gen_delims_byte(id.as_bytes()[id.len() - 1])
                {
                    definition.set_prefix(true);
                }
            }
            // Step 16.2
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            v => {
                return Err(ErrorCode::InvalidIriMapping
                    .and_source(anyhow!("Expected string as @id but got {:?}", v)))
            }
        }
    }
    // Step 17-20
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    match to_prefix_and_suffix(term) {
        Some((prefix, suffix)) => {
            // Step 17.1
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            if is_compact_iri(term) && local_context.contains_key(prefix) {
                // TODO: Should optional params be default or same as callee?
                create_term_definition(
                    processor,
                    active_context,
                    local_context,
                    prefix,
                    defined,
                    optional,
                )?;
            }
            // Step 17.2
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            if let Some(prefix_iri) = active_context.term_definition(prefix).map(Definition::iri) {
                definition.set_iri(format!("{}{}", prefix_iri, suffix));
            }
            // Step 17.3
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            // NOTE: See <https://github.com/w3c/json-ld-api/issues/195>.
            //assert!(is_absolute_or_blank_node_ident(term));
            definition.set_iri(term);
        }
        // Step 18-20
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        None => {
            // Step 18
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            if term.contains('/') {
                // Step 18.1: Term is a relative IRI.
                // Step 18.2
                let resolved = ExpandIriOptions::constant(active_context)
                    .vocab(true)
                    .expand_str(processor, term)?
                    .ok_or_else(|| {
                        ErrorCode::InvalidIriMapping.and_source(anyhow!(
                            "Expected an absolute IRI reference as resolved term, \
                             but got null: term={:?}",
                            term
                        ))
                    })?;
                if !is_absolute_iri(&resolved) {
                    return Err(ErrorCode::InvalidIriMapping.and_source(anyhow!(
                        "Expected an absolute IRI reference as resolved term, \
                         but got {:?}: term={:?}",
                        resolved,
                        term
                    )));
                } else {
                    definition.set_iri(resolved);
                }
            } else if term == "@type" {
                // Step 19
                // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                // as WD-json-ld11-api-20191018 has ambiguity.
                definition.set_iri("@type");
            } else if let Some(vocab) = active_context.vocab() {
                // Step 20
                // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                // as WD-json-ld11-api-20191018 has ambiguity.
                definition.set_iri(format!("{}{}", vocab, term));
            } else {
                // Step 20
                // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                // as WD-json-ld11-api-20191018 has ambiguity.
                return Err(ErrorCode::InvalidIriMapping.and_source(anyhow!(
                    "term={:?}, active context has no vocab mapping",
                    term
                )));
            }
        }
    }
    // Step 21
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(container) = value.get("@container") {
        // Step 21.1
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        let container = validate_container_non_reverse(container)?;
        // Step 21.2
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        if processor.is_processing_mode_1_0() {
            let item = match container {
                Container::Single(v) => v,
                Container::Array(arr) => {
                    return Err(ErrorCode::InvalidContainerMapping.and_source(anyhow!(
                        "Unexpected `@container` value {:?} with processing mode `json-ld-1.0`",
                        arr
                    )))
                }
            };
            match item {
                ContainerItem::Graph | ContainerItem::Id | ContainerItem::Type => {}
                v => {
                    return Err(ErrorCode::InvalidContainerMapping.and_source(anyhow!(
                        "Unexpected @container value {:?} with processing mode `json-ld-1.0`",
                        v
                    )))
                }
            }
        }
        // Step 21.3
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        definition.set_container(Nullable::Value(container.clone()));
        // Step 21.4
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        if definition.container_contains(ContainerItem::Type) {
            match definition.ty() {
                None => {
                    // Step 21.4.1
                    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                    // as WD-json-ld11-api-20191018 has ambiguity.
                    definition.set_ty("@id");
                }
                // Step 21.4.2
                // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                // as WD-json-ld11-api-20191018 has ambiguity.
                Some("@id") | Some("@vocab") => {}
                Some(ty) => {
                    // Step 21.4.2
                    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
                    // as WD-json-ld11-api-20191018 has ambiguity.
                    return Err(ErrorCode::InvalidTypeMapping.and_source(anyhow!(
                        "container = {:?}, type = {:?}",
                        container,
                        ty
                    )));
                }
            }
        }
    }
    // Step 22
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(index) = value.get("@index") {
        // Step 22.1
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        if processor.is_processing_mode_1_0()
            || !definition.container_contains(ContainerItem::Index)
        {
            let processing_mode = if processor.is_processing_mode_1_0() {
                "json-ld-1.0"
            } else {
                "not json-ld-1.0"
            };
            return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                "`value` has `@index` entry, processing mode is {}, container = {:?}",
                processing_mode,
                definition.container()
            )));
        }
        // Step 22.2
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        let index = match index {
            Value::String(s) => s,
            v => {
                return Err(ErrorCode::InvalidTermDefinition
                    .and_source(anyhow!("Invalid `@index` value {:?}", v)))
            }
        };
        // Step 22.3
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        definition.set_index(index);
    }
    // Step 23
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(context) = value.get("@context") {
        // Step 23.1
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                "`value` has `@context` entry but processing mode is json-ld-1.0"
            )));
        }
        // Step 23.2: `context` is already the value associated with the `@context` entry.
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        // Step 23.3
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        // FIXME: Invoke context processing algorithm. Result might not be `Value`.
        let context: Context =
            unimplemented!("Invoke context processing algorithm: context={:?}", context);
        // Step 23.4
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        definition.set_local_context(context);
    }
    // Step 24
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(language) = value.get("@language") {
        if !value.contains_key("@type") {
            // Step 24.1
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            let language = match language {
                Value::Null => Nullable::Null,
                Value::String(s) => Nullable::Value(s.as_str()),
                v => {
                    return Err(ErrorCode::InvalidLanguageMapping.and_source(anyhow!(
                        "Expected string or null as `@language` value, but got {:?}",
                        v
                    )))
                }
            };
            // TODO: Issue a warning if `language` is not well-formed according to section 2.2.9 of BCP47.
            // Step 24.2
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            // TODO: Processors MAY normalize language tags to lower case.
            definition.set_language(language.map(ToOwned::to_owned));
        }
    }
    // Step 25
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(direction) = value.get("@direction") {
        if !value.contains_key("@type") {
            // Step 25.1
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            // FIXME: Create `Direction` type.
            let direction = Nullable::<Direction>::try_from(direction)
                .map_err(|e| ErrorCode::InvalidBaseDirection.and_source(e))?;
            // Step 25.2
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            definition.set_direction(direction);
        }
    }
    // Step 26
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(nest) = value.get("@nest") {
        // Step 26.1
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                "Found `@nest` but processing mode is `json-ld-1.0`"
            )));
        }
        // Step 26.2
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        let nest = match nest {
            Value::String(s) => s.as_str(),
            v => {
                return Err(ErrorCode::InvalidNestValue
                    .and_source(anyhow!("Expected string but got {:?}", v)))
            }
        };
        if nest != "@nest" && processor.is_keyword(nest) {
            return Err(ErrorCode::InvalidNestValue
                .and_source(anyhow!("Got a keyword {:?} other than `\"@nest\"`", nest)));
        }
        definition.set_nest(nest);
    }
    // Step 27
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(prefix) = value.get("@prefix") {
        // Step 27.1
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                "Found `@prefix` but processing mode is `json-ld-1.0`"
            )));
        } else if term.contains(':') || term.contains('/') {
            return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                "Found `@prefix` but the term {:?} contains colon or slash",
                term
            )));
        }
        // Step 27.2
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        let prefix = match prefix {
            Value::Bool(v) => *v,
            v => {
                return Err(ErrorCode::InvalidPrefixValue
                    .and_source(anyhow!("Expected boolean but got {:?}", v)))
            }
        };
        definition.set_prefix(prefix);
        // Step 27.3
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        if prefix && processor.is_keyword(definition.iri()) {
            return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                "`prefix` flag is set to `true` for a definition \
                 whose IRI mapping is a keyword {:?}",
                definition.iri()
            )));
        }
    }
    // Step 28
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    for key in value.keys() {
        match key.as_str() {
            "@id" | "@reverse" | "@container" | "@context" | "@language" | "@nest" | "@prefix"
            | "@type" => {}
            v => {
                return Err(ErrorCode::InvalidTermDefinition
                    .and_source(anyhow!("Unexpected entry: key={:?}", v)))
            }
        }
    }
    // Step 29
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    // TODO: How to treat if `previous_definition` is `null`?
    let mut new_definition = None;
    if let Some(previous_definition) =
        previous_definition.and_then(Into::<Option<Definition>>::into)
    {
        if !optional.override_protected && previous_definition.is_protected() {
            // Step 29.1
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            if !definition.is_same_other_than_protected(&previous_definition) {
                return Err(ErrorCode::ProtectedTermRedefinition.into());
            }
            // Step 29.2
            // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
            // as WD-json-ld11-api-20191018 has ambiguity.
            new_definition = Some(previous_definition);
        }
    }
    let definition = match new_definition {
        Some(v) => v,
        None => definition.build(),
    };
    // Step 30
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    active_context
        .term_definitions
        .insert(term.to_owned(), Nullable::Value(definition));
    defined.insert(term.to_owned(), true);

    Ok(())
}

/// Validates `@container` value.
///
/// Returns `Ok(container)` if the value is valid, `Err(_)` otherwise.
fn validate_container_non_reverse(container: &Value) -> Result<Container> {
    let container = Container::try_from(container)
        .map_err(|e| ErrorCode::InvalidContainerMapping.and_source(e))?;
    let arr = match container {
        Container::Single(_) => {
            // > either `@graph`, `@id`, `@index`, `@language`, `@list`, `@set`, `@type`
            return Ok(container);
        }
        Container::Array(ref arr) => arr,
    };

    if arr.len() == 1 {
        // > an array containing exactly any one of those keywords
        return Ok(container);
    }

    {
        let mut has_graph = false;
        let mut has_id = false;
        let mut has_index = false;
        for item in arr {
            match item {
                ContainerItem::Graph => has_graph = true,
                ContainerItem::Id => has_id = true,
                ContainerItem::Index => has_index = true,
                ContainerItem::Set => {}
                v => {
                    return Err(ErrorCode::InvalidContainerMapping.and_source(anyhow!(
                        "Unexpected item {:?} in container {:?}",
                        v,
                        arr
                    )))
                }
            }
        }
        if has_graph && (has_id ^ has_index) {
            // an array containing `@graph` and either `@id` or `@index` optionally including `@set`
            return Ok(container);
        }
    }

    {
        let mut has_set = false;
        for item in arr {
            match item {
                ContainerItem::Set => has_set = true,
                ContainerItem::Index
                | ContainerItem::Id
                | ContainerItem::Type
                | ContainerItem::Language => {}
                v => {
                    return Err(ErrorCode::InvalidContainerMapping.and_source(anyhow!(
                        "Unexpected item {:?} in container {:?}",
                        v,
                        arr
                    )))
                }
            }
        }
        if has_set {
            // > an array containing a combination of `@set` and any of
            // > `@index`, `@id`, `@type`, `@language` in any order
            return Ok(container);
        }
    }

    Err(ErrorCode::InvalidContainerMapping.and_source(anyhow!("Unexpected container {:?}", arr)))
}
