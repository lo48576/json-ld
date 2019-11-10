//! Part of create term definition algorithm, for non-reverse term definitions.

use std::{collections::HashMap, convert::TryFrom};

use anyhow::anyhow;
use serde_json::{Map as JsonMap, Value};

use crate::{
    context::{
        create_term_def::{create_term_definition, OptionalParams},
        definition::{Container, ContainerItem, Definition, DefinitionBuilder, Direction},
        Context,
    },
    error::{ErrorCode, Result},
    expand::iri::ExpandIriOptions,
    iri::{
        is_absolute_iri, is_absolute_or_blank_node_ident, is_compact_iri, is_gen_delims_byte,
        to_prefix_and_suffix,
    },
    json::Nullable,
    processor::{Processor, ProcessorOptions},
    remote::LoadRemoteDocument,
};

/// Runs rest of the create term definition algorithm for the case `@reverse` exists.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#create-term-definition>
// Step 15 and after.
// NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
// as WD-json-ld11-api-20191018 has ambiguity.
#[allow(clippy::too_many_arguments)] // TODO: FIXME
pub(crate) async fn run_for_non_reverse<L: LoadRemoteDocument>(
    processor: &Processor<L>,
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
    // Step 16-20
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    process_iri(
        processor,
        active_context,
        local_context,
        term,
        defined,
        optional,
        value,
        &mut definition,
        simple_term,
    )
    .await?;
    // Step 21
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    process_container(processor, value, &mut definition).await?;
    // Step 22
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    process_index(processor.options(), value, &mut definition)?;
    // Step 23
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    process_local_context(processor, active_context, value, &mut definition).await?;
    // Step 24
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    process_language(value, &mut definition)?;
    // Step 25
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    process_direction(value, &mut definition)?;
    // Step 26
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    process_nest(processor.options(), value, &mut definition)?;
    // Step 27
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    process_prefix(processor.options(), term, value, &mut definition)?;
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
    let definition = build_term_definition(optional, definition, previous_definition)?;
    // Step 30
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    active_context
        .term_definitions
        .insert(term.to_owned(), Nullable::Value(definition));
    defined.insert(term.to_owned(), true);

    Ok(())
}

/// Processes the language mapping.
fn process_language(
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
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

    Ok(())
}

/// Processes the IRI mapping.
#[allow(clippy::too_many_arguments)] // TODO: FIXME
async fn process_iri<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &mut Context,
    local_context: &JsonMap<String, Value>,
    term: &str,
    defined: &mut HashMap<String, bool>,
    optional: OptionalParams,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
    simple_term: bool,
) -> Result<()> {
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
                // > If value contains the `@id` entry is `null`, the term is not used for IRI
                // > expansion, but is retained to be able to detect future redefinitions of this term.
                unimplemented!("TODO: What to do if `@id` is null?");
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
                    .expand_str(processor, id)
                    .await?
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
                )
                .await?;
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
                    .expand_str(processor, term)
                    .await?
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

    Ok(())
}

/// Processes the container mapping.
async fn process_container<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
    // Step 21
    // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
    // as WD-json-ld11-api-20191018 has ambiguity.
    if let Some(container) = value.get("@container") {
        // Step 21.1
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        let container = validate_container_non_reverse(container).await?;
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

    Ok(())
}

/// Processes the index mapping.
fn process_index(
    processor: &ProcessorOptions,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
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

    Ok(())
}

/// Processes the local context.
async fn process_local_context<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &mut Context,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
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
        let context: Context = active_context
            .join(processor, context, true)
            .await
            .map_err(|e| ErrorCode::InvalidScopedContext.and_source(e))?;
        // Step 23.4
        // NOTE: Using <https://pr-preview.s3.amazonaws.com/w3c/json-ld-api/pull/182.html#create-term-definition>
        // as WD-json-ld11-api-20191018 has ambiguity.
        definition.set_local_context(context);
    }

    Ok(())
}

/// Processes the direction mapping.
fn process_direction(
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
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

    Ok(())
}

/// Processes the nest value.
fn process_nest(
    processor: &ProcessorOptions,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
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

    Ok(())
}

/// Processes the prefix flag.
fn process_prefix(
    processor: &ProcessorOptions,
    term: &str,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
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

    Ok(())
}

/// Checks the override protected flag and builds the term definition.
fn build_term_definition(
    optional: OptionalParams,
    definition: DefinitionBuilder,
    previous_definition: Option<Nullable<Definition>>,
) -> Result<Definition> {
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
    Ok(match new_definition {
        Some(v) => v,
        None => definition.build(),
    })
}

/// Validates `@container` value.
///
/// Returns `Ok(container)` if the value is valid, `Err(_)` otherwise.
async fn validate_container_non_reverse(container: &Value) -> Result<Container> {
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
