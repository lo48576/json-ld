//! Processing function for a context definition.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::Arc,
};

use anyhow::anyhow;
use iri_string::types::{IriReferenceStr, IriStr, IriString, RelativeIriStr};
use serde_json::{Map as JsonMap, Value};

use crate::{
    context::{
        create_term_def::{create_term_definition, OptionalParams},
        definition::Direction,
        Context,
    },
    error::{ErrorCode, Result},
    expand::iri::ExpandIriOptions,
    json::Nullable,
    processor::{Processor, ProcessorOptions},
    remote::{LoadDocumentOptions, LoadRemoteDocument, Profile, RemoteDocument},
};

/// Processes single context which is a map.
pub(crate) async fn process_context_definition<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &Context,
    remote_contexts: &mut HashSet<IriString>,
    propagate: bool,
    mut result: Context,
    context: &JsonMap<String, Value>,
) -> Result<Context> {
    // Step 5.4: Otherwise, _context_ is a context definition.
    // Step 5.5
    process_ctxdef_version(processor.options(), context)?;
    // Step 5.6
    let context: Cow<JsonMap<String, Value>> =
        process_ctxdef_import(processor, active_context, context).await?;
    // Step 5.7
    process_ctxdef_base(remote_contexts, &mut result, &context)?;
    // Step 5.8
    process_ctxdef_vocab(processor, &mut result, &context).await?;
    // Step 5.9.
    process_ctxdef_language(&mut result, &context)?;
    // Step 5.10.
    process_ctxdef_direction(processor.options(), &mut result, &context)?;
    // Step 5.11.
    // Note that this does only error handling.
    process_ctxdef_propagate(processor.options(), &context)?;
    // Step 5.12.
    let mut defined = HashMap::new();
    // Step 5.13.
    let protected = match context.get("@protected") {
        None => None,
        Some(Value::Bool(v)) => Some(*v),
        Some(v) => {
            return Err(ErrorCode::Uncategorized
                .and_source(anyhow!("Expected boolean as `@protected`, but got {:?}", v)))
        }
    };
    let options = OptionalParams::new()
        .propagate(propagate)
        .protected_opt(protected);
    for key in context.keys().map(String::as_str) {
        match key {
            "@base" | "@direction" | "@import" | "@language" | "@propagate" | "@protected"
            | "@version" | "@vocab" => continue,
            _ => {}
        }
        create_term_definition(processor, &mut result, &context, key, &mut defined, options)
            .await?;
    }

    Ok(result)
}

/// Processes `@version` entry of the context definition.
fn process_ctxdef_version(
    processor: &ProcessorOptions,
    context: &JsonMap<String, Value>,
) -> Result<()> {
    // Step 5.5
    if let Some(version) = context.get("@version") {
        // Step 5.5.1
        let is_1_1 = version.as_f64().map_or(false, |v| v >= 1.09 && v <= 1.11);
        if !is_1_1 {
            return Err(
                ErrorCode::InvalidVersionValue.and_source(anyhow!("`@version` = {:?}", version))
            );
        }
        // Step 5.5.2
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::ProcessingModeConflict.and_source(anyhow!(
                "Got `@version` = 1.1, but processing mode is `json-ld-1.0`"
            )));
        }
    }

    Ok(())
}

/// Processes `@import` entry of the context definition.
async fn process_ctxdef_import<'a, L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &Context,
    context: &'a JsonMap<String, Value>,
) -> Result<Cow<'a, JsonMap<String, Value>>> {
    // Step 5.6
    let import = match context.get("@import") {
        Some(v) => v,
        None => return Ok(Cow::Borrowed(context)),
    };

    // Step 5.6.1
    if processor.is_processing_mode_1_0() {
        return Err(ErrorCode::InvalidContextEntry.and_source(anyhow!(
            "Found `@import` but processing mode is `json-ld-1.0`"
        )));
    }
    // Step 5.6.2
    let import = import.as_str().ok_or_else(|| {
        ErrorCode::InvalidImportValue.and_source(anyhow!("Expected string but got {:?}", import))
    })?;
    // Step 5.6.3
    let import = {
        let base = match processor.base(&active_context) {
            Some(v) => v,
            None => unimplemented!("FIXME: What to do if no base IRI available?"),
        };
        let import = IriReferenceStr::new(import).map_err(|e| {
            ErrorCode::Uncategorized.and_source(e).context(format!(
                "Cannot resolve `@import` IRI ({:?}) because it is not an IRI reference",
                import
            ))
        })?;
        import.resolve_against(base.to_absolute())
    };
    // Step 5.6.4, 5.6.5
    // NOTE: The spec does not say this should be cached (but also does not say this should not
    // be cached...
    let remote_doc: Arc<RemoteDocument> = {
        let mut load_opts = LoadDocumentOptions::new();
        load_opts.set_profile(Profile::Context);
        load_opts.set_request_profile(Profile::Context);
        processor
            .loader()
            .load(&import, load_opts)
            .await
            .map_err(|e| {
                ErrorCode::LoadingRemoteContextFailed
                    .and_source(e)
                    .context("Failed to dereference `@import`")
            })?
    };
    // Step 5.6.6
    let import_context = match remote_doc.document().get("@context") {
        Some(Value::Object(map)) => map,
        Some(v) => {
            return Err(ErrorCode::InvalidRemoteContext.and_source(anyhow!(
                "Expected a map as `@context` entry in remote doc \
                 specified by `@import`, but got {:?}",
                v
            )))
        }
        None => {
            return Err(ErrorCode::InvalidRemoteContext.and_source(anyhow!(
                "`@context` entry not found in remote doc specified by `@import`"
            )))
        }
    };
    // Step 5.6.7
    if import_context.contains_key("@import") {
        return Err(ErrorCode::InvalidContextEntry.and_source(anyhow!(
            "`@import` entry found in the remote doc specified by `@import`"
        )));
    };
    // Step 5.6.8
    if import_context.is_empty() {
        return Ok(Cow::Borrowed(context));
    }
    let mut context = context.clone();
    for (k, v) in import_context {
        // NOTE: Entry API (`context.entry(k.clone())`) requires the key to be owned.
        // To avoid cloning, use `.contains_key(v)` which causes double lookup when the entry is
        // missing.
        // NOTE: Raw entry API <https://github.com/rust-lang/rust/issues/56167> is perfect for this
        // case, but it is currently (at Rust 1.39) unstable in std, and also is not available in
        // `serde_json::Map`.
        if !context.contains_key(k) {
            context.insert(k.clone(), v.clone());
        }
    }

    Ok(Cow::Owned(context))
}

/// Processes `@base` entry of the context definition.
fn process_ctxdef_base(
    remote_contexts: &HashSet<IriString>,
    result: &mut Context,
    context: &JsonMap<String, Value>,
) -> Result<()> {
    // Step 5.7
    if let Some(value) = context.get("@base") {
        if remote_contexts.is_empty() {
            let base = process_ctxdef_base_impl(result, value)?;
            result.set_base(base);
        }
    }

    Ok(())
}

/// Internal implementation of `process_ctxdef_base()`.
fn process_ctxdef_base_impl(result: &Context, value: &Value) -> Result<Nullable<IriString>> {
    // Step 5.7.1: Initialize _value_ to the value associated with the `@base` entry.
    // Step 5.7.2-5.7.5
    match value {
        // Step 5.7.2
        Value::Null => Ok(Nullable::Null),
        // Step 5.7.3-5.7.5
        Value::String(value) => {
            // Step 5.7.3
            if let Ok(value) = IriStr::new(value) {
                return Ok(Nullable::Value(value.to_owned()));
            }
            // Step 5.7.4
            if let Ok(value) = RelativeIriStr::new(value) {
                if let Nullable::Value(result_base) = result.base() {
                    let resolved = value.resolve_against(result_base.to_absolute());
                    return Ok(Nullable::Value(resolved.to_owned()));
                } else {
                    // Step 5.7.5
                    return Err(ErrorCode::InvalidBaseIri.and_source(anyhow!(
                        "Got a relative IRI reference {:?} as `@base`, \
                         but base IRI of `result` is not available",
                        value
                    )));
                }
            }
            // Step 5.7.5
            Err(ErrorCode::InvalidBaseIri.and_source(anyhow!(
                "Value of `@base` ({:?}) is not an IRI reference",
                value
            )))
        }
        // Step 5.7.5
        v => Err(ErrorCode::InvalidBaseIri.and_source(anyhow!(
            "Expected `null` or a string as `@base`, but got {:?}",
            v
        ))),
    }
}

/// Processes `@vocab` entry of the context definition.
async fn process_ctxdef_vocab<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    result: &mut Context,
    context: &JsonMap<String, Value>,
) -> Result<()> {
    // Step 5.8
    if let Some(value) = context.get("@vocab") {
        // Step 5.8.1: Initialize _value_ to the value associated with the `@base` entry.
        // Step 5.8.2, 5.8.3
        let value = match value {
            // Step 5.8.2
            Value::Null => {
                result.set_vocab(Nullable::Null);
                return Ok(());
            }
            // Step 5.8.3
            Value::String(s) => s,
            // Step 5.8.3
            v => {
                return Err(ErrorCode::InvalidVocabMapping
                    .and_source(anyhow!("Expected string as `@vocab`, but got {:?}", v)))
            }
        };
        // Step 5.8.3
        if value.starts_with("_:") || IriStr::new(value).is_ok() {
            let expanded = ExpandIriOptions::constant(result)
                .vocab(true)
                .document_relative(true)
                .expand_str(processor, value)
                .await?
                .map(Cow::into_owned);
            result.set_vocab(expanded);
        } else {
            return Err(ErrorCode::InvalidVocabMapping.and_source(anyhow!(
                "Expected blank node identifier or an IRI, but got {:?}",
                value
            )));
        }
    }

    Ok(())
}

/// Processes `@language` entry of the context definition.
fn process_ctxdef_language(result: &mut Context, context: &JsonMap<String, Value>) -> Result<()> {
    // Step 5.9.
    if let Some(value) = context.get("@language") {
        // Step 5.9.1: Initialize _value_ to the value associated with the `@language` entry.
        // Step 5.9.2, 5.9.3
        match value {
            // Step 5.9.2
            Value::Null => result.set_default_language(None),
            // Step 5.9.3
            Value::String(value) => {
                // TODO: Emit a warning if the value is not a well-formed language tag.
                // NOTE: The spec says "Processors MAY normalize language tags to lower case".
                result.set_default_language(Some(value.into()));
            }
            // Step 5.9.3
            v => {
                return Err(ErrorCode::InvalidDefaultLanguage.and_source(anyhow!(
                    "Expected `null` or string as `@language`, but got {:?}",
                    v
                )))
            }
        }
    }

    Ok(())
}

/// Processes `@direction` entry of the context definition.
fn process_ctxdef_direction(
    processor: &ProcessorOptions,
    result: &mut Context,
    context: &JsonMap<String, Value>,
) -> Result<()> {
    // Step 5.10.
    if let Some(value) = context.get("@direction") {
        // Step 5.10.1
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::InvalidContextEntry.and_source(anyhow!(
                "Found `@direction` while processing mode is `json-ld-1.0`"
            )));
        }
        // Step 5.10.2: Initialize _value_ to the value associated with the `@direction` entry.
        // Step 5.10.3, 5.10.4
        let value: Nullable<Direction> = value
            .try_into()
            .map_err(|e| ErrorCode::InvalidBaseDirection.and_source(e))?;
        result.set_default_base_direction(value.into());
    }

    Ok(())
}

/// Processes `@propagate` entry of the context definition.
fn process_ctxdef_propagate(
    processor: &ProcessorOptions,
    context: &JsonMap<String, Value>,
) -> Result<()> {
    // Step 5.11.
    if let Some(value) = context.get("@direction") {
        // Step 5.11.1
        if processor.is_processing_mode_1_0() {
            return Err(ErrorCode::InvalidContextEntry.and_source(anyhow!(
                "Found `@propagate` while processing mode is `json-ld-1.0`"
            )));
        }
        // Step 5.11.2
        if !value.is_boolean() {
            return Err(ErrorCode::InvalidPropagateValue.and_source(anyhow!(
                "Expected boolean as `@propagate` but got {:?}",
                value
            )));
        }
    }

    Ok(())
}
