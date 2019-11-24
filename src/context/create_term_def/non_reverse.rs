//! Part of create term definition algorithm, for non-reverse term definitions.

use std::{collections::HashMap, convert::TryFrom};

use anyhow::anyhow;
use serde_json::{Map as JsonMap, Value};

use crate::{
    context::{
        create_term_def::{create_term_definition, OptionalParams},
        definition::{Container, ContainerItem, Definition, DefinitionBuilder, Direction},
        Context, ValueWithBase,
    },
    error::{ErrorCode, Result},
    expand::iri::ExpandIriOptions,
    iri::{
        is_absolute_iri_ref, is_absolute_ref_or_blank_node_ident, is_compact_iri,
        is_gen_delims_byte, to_prefix_and_suffix,
    },
    json::Nullable,
    processor::{Processor, ProcessorOptions},
    remote::LoadRemoteDocument,
    syntax::has_form_of_keyword,
};

/// Runs rest of the create term definition algorithm for the case `@reverse` exists.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#create-term-definition>
// Step 15-
#[allow(clippy::too_many_arguments)] // TODO: FIXME
pub(crate) async fn run_for_non_reverse<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &mut Context,
    local_context: ValueWithBase<'_, &JsonMap<String, Value>>,
    term: &str,
    defined: &mut HashMap<String, bool>,
    optional: OptionalParams,
    value: &JsonMap<String, Value>,
    mut definition: DefinitionBuilder,
    previous_definition: Option<Definition>,
    simple_term: bool,
) -> Result<()> {
    // Step 15
    definition.set_reverse(false);
    // Step 16-20
    let process_iri_status = process_iri(
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
    if process_iri_status == ProcessIriStatus::Stop {
        return Ok(());
    }
    // Step 21
    process_container(processor, value, &mut definition).await?;
    // Step 22
    process_index(processor.options(), value, &mut definition)?;
    // Step 23
    process_local_context(
        processor,
        active_context,
        local_context.with_new_value(value),
        &mut definition,
    )
    .await?;
    // Step 24
    process_language(value, &mut definition)?;
    // Step 25
    process_direction(value, &mut definition)?;
    // Step 26
    process_nest(processor.options(), value, &mut definition)?;
    // Step 27
    process_prefix(processor.options(), term, value, &mut definition)?;
    // Step 28
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
    let definition = build_term_definition(optional, definition, previous_definition)?;
    // Step 30
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
    if let Some(language) = value.get("@language") {
        if !value.contains_key("@type") {
            // Step 24.1
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
            // TODO: Processors MAY normalize language tags to lower case.
            definition.set_language(language.map(ToOwned::to_owned));
        }
    }

    Ok(())
}

/// Status of IRI processing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProcessIriStatus {
    /// Continue running following step.
    Continue,
    /// Not error, but stop the processing and return from the context definition algorithm.
    Stop,
}

/// Processes the IRI mapping.
#[allow(clippy::too_many_arguments)] // TODO: FIXME
async fn process_iri<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &mut Context,
    local_context: ValueWithBase<'_, &JsonMap<String, Value>>,
    term: &str,
    defined: &mut HashMap<String, bool>,
    optional: OptionalParams,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
    simple_term: bool,
) -> Result<ProcessIriStatus> {
    // Step 16
    if let Some(id) = value.get("@id").filter(|id| id.as_str() != Some(term)) {
        match id {
            // Step 16.1
            Value::Null => {
                // > If value contains the `@id` entry is `null`, the term is not used for IRI
                // > expansion, but is retained to be able to detect future redefinitions of this term.
                //
                // This seems essentially not changed from JSON-LD-API 1.0, so return from this
                // function here.
                active_context
                    .term_definitions
                    .insert(term.to_owned(), Nullable::Null);
            }
            // Step 16.3-
            Value::String(id) => {
                // Step 16.3
                if !processor.is_keyword(id) && has_form_of_keyword(id) {
                    // TODO: Generate warning.
                    return Ok(ProcessIriStatus::Stop);
                }
                // Step 16.4
                let id = ExpandIriOptions::mutable(active_context, local_context, defined)
                    .vocab(true)
                    .expand_str(processor, id)
                    .await?
                    .ok_or_else(|| {
                        ErrorCode::InvalidIriMapping
                            .and_source(anyhow!("@id ({:?}) is expanded to `null`", id))
                    })?;
                if !processor.is_keyword(&id) && !is_absolute_ref_or_blank_node_ident(&id) {
                    return Err(ErrorCode::InvalidIriMapping.and_source(anyhow!(
                        "@id ({:?}) should be a keyword, \
                         an IRI (which is absolute), or a blank node identifier",
                        id
                    )));
                } else if id == "@context" {
                    return Err(ErrorCode::InvalidKeywordAlias
                        .and_source(anyhow!("Invalid alias to `@context`")));
                }
                definition.set_iri(id);
                let id = definition.iri();
                // Step 16.5
                if (!term.is_empty() && term[1..(term.len() - 1)].contains(':'))
                    || term.contains('/')
                {
                    let expanded =
                        ExpandIriOptions::mutable(active_context, local_context, defined)
                            .vocab(true)
                            .expand_str(processor, term)
                            .await?;
                    if expanded.as_ref().map(|s| &**s) != Some(id) {
                        return Err(ErrorCode::InvalidIriMapping.and_source(anyhow!(
                            "expanded={:?}, term={:?}",
                            expanded,
                            term
                        )));
                    }
                }
                // Step 16.6
                if !term.contains(':')
                    && !term.contains('/')
                    && simple_term
                    && is_gen_delims_byte(id.as_bytes()[id.len() - 1])
                {
                    definition.set_prefix(true);
                }
            }
            // Step 16.2
            v => {
                return Err(ErrorCode::InvalidIriMapping
                    .and_source(anyhow!("Expected string as @id but got {:?}", v)))
            }
        }

        return Ok(ProcessIriStatus::Continue);
    }
    // Step 17-20
    match to_prefix_and_suffix(term) {
        Some((prefix, suffix)) => {
            debug_assert!(!prefix.is_empty());
            // Step 17.1
            if is_compact_iri(term) && local_context.value().contains_key(prefix) {
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
            if let Some(prefix_iri) = active_context.term_definition(prefix).map(Definition::iri) {
                definition.set_iri(format!("{}{}", prefix_iri, suffix));
            } else {
                // Step 17.3
                definition.set_iri(term);
            }
        }
        // Step 18-20
        _ => {
            // Step 18
            if term.contains('/') {
                // Step 18.1: Term is a relative IRI reference.
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
                if !is_absolute_iri_ref(&resolved) {
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
                definition.set_iri("@type");
            } else if let Nullable::Value(vocab) = active_context.vocab() {
                // Step 20
                definition.set_iri(format!("{}{}", vocab, term));
            } else {
                // Step 20
                return Err(ErrorCode::InvalidIriMapping.and_source(anyhow!(
                    "term={:?}, active context has no vocab mapping",
                    term
                )));
            }
        }
    }

    Ok(ProcessIriStatus::Continue)
}

/// Processes the container mapping.
async fn process_container<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    value: &JsonMap<String, Value>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
    // Step 21
    if let Some(container_raw) = value.get("@container") {
        let has_array_form = container_raw.is_array();
        // Step 21.1
        let container = validate_container_non_reverse(container_raw).await?;
        // Step 21.2
        if processor.is_processing_mode_1_0() {
            if has_array_form {
                return Err(ErrorCode::InvalidContainerMapping.and_source(anyhow!(
                    "Expected `@container` to be a string but got {:?}, \
                     with processing mode `json-ld-1.0`",
                    container_raw
                )));
            }
            match container.get_single_item() {
                Some(item @ ContainerItem::Graph)
                | Some(item @ ContainerItem::Id)
                | Some(item @ ContainerItem::Type) => {
                    return Err(ErrorCode::InvalidContainerMapping.and_source(anyhow!(
                        "Unexpected `@container` value {:?} with processing mode `json-ld-1.0`",
                        item
                    )))
                }
                _ => {}
            }
        }
        // Step 21.3
        definition.set_container(Nullable::Value(container));
        // Step 21.4
        if definition.container_contains(ContainerItem::Type) {
            match definition.ty() {
                None => {
                    // Step 21.4.1
                    definition.set_ty("@id");
                }
                // Step 21.4.2
                Some("@id") | Some("@vocab") => {}
                Some(ty) => {
                    // Step 21.4.2
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
    if let Some(index) = value.get("@index") {
        // Step 22.1
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
        let index = index.as_str().ok_or_else(|| {
            ErrorCode::InvalidTermDefinition
                .and_source(anyhow!("Invalid `@index` value {:?}", index))
        })?;
        // TODO: Now `index` must be a string expanding to an absolute IRI. How to check that?
        // Step 22.3
        definition.set_index(index);
    }

    Ok(())
}

/// Processes the local context.
async fn process_local_context<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &mut Context,
    value: ValueWithBase<'_, &JsonMap<String, Value>>,
    definition: &mut DefinitionBuilder,
) -> Result<()> {
    // Step 23
    if let Some(context) = value.value().get("@context") {
        // Step 23.1
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                "`value` has `@context` entry but processing mode is json-ld-1.0"
            )));
        }
        // Step 23.2: `context` is already the value associated with the `@context` entry.
        // Step 23.3
        let context: Context = active_context
            .join_context_value(processor, context, value.base(), true)
            .await
            .map_err(|e| ErrorCode::InvalidScopedContext.and_source(e))?;
        // Step 23.4
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
    if let Some(direction) = value.get("@direction") {
        if !value.contains_key("@type") {
            // Step 25.1
            let direction = Nullable::<Direction>::try_from(direction)
                .map_err(|e| ErrorCode::InvalidBaseDirection.and_source(e))?;
            // Step 25.2
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
    if let Some(nest) = value.get("@nest") {
        // Step 26.1
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::InvalidTermDefinition.and_source(anyhow!(
                "Found `@nest` but processing mode is `json-ld-1.0`"
            )));
        }
        // Step 26.2
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
    if let Some(prefix) = value.get("@prefix") {
        // Step 27.1
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
        let prefix = match prefix {
            Value::Bool(v) => *v,
            v => {
                return Err(ErrorCode::InvalidPrefixValue
                    .and_source(anyhow!("Expected boolean but got {:?}", v)))
            }
        };
        definition.set_prefix(prefix);
        // Step 27.3
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
    previous_definition: Option<Definition>,
) -> Result<Definition> {
    // Step 29
    let definition = definition.build();
    if let Some(previous_definition) = previous_definition {
        if !optional.override_protected && previous_definition.is_protected() {
            // Step 29.1
            if !definition.eq_other_than_protected(&previous_definition) {
                return Err(ErrorCode::ProtectedTermRedefinition.into());
            }
            // Step 29.2
            return Ok(previous_definition);
        }
    }

    Ok(definition)
}

/// Returns the `@container` value, if validated.
///
/// Returns `Ok(container)` if the value is valid, `Err(_)` otherwise.
// Step 21.
async fn validate_container_non_reverse(container: &Value) -> Result<Container> {
    let container = Container::try_from(container)
        .map_err(|e| ErrorCode::InvalidContainerMapping.and_source(e))?;
    if container.len() == 1 {
        // > either `@graph`, `@id`, `@index`, `@language`, `@list`, `@set`, `@type`,
        // > or an array containing exactly any one of those keywords
        return Ok(container);
    }

    {
        let mut has_graph = false;
        let mut has_id = false;
        let mut has_index = false;
        for item in container.iter() {
            match item {
                ContainerItem::Graph => has_graph = true,
                ContainerItem::Id => has_id = true,
                ContainerItem::Index => has_index = true,
                ContainerItem::Set => {}
                v => {
                    return Err(ErrorCode::InvalidContainerMapping.and_source(anyhow!(
                        "Unexpected item {:?} in container {:?}",
                        v,
                        container
                    )))
                }
            }
        }
        if has_graph && (has_id ^ has_index) {
            // > an array containing `@graph` and either `@id` or `@index` optionally including
            // > `@set`
            return Ok(container);
        }
    }

    {
        let mut has_set = false;
        for item in container.iter() {
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
                        container
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

    Err(ErrorCode::InvalidContainerMapping
        .and_source(anyhow!("Unexpected container {:?}", container)))
}
