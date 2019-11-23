//! Context processing algorithm.

use std::{
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use anyhow::anyhow;
use iri_string::types::{IriReferenceStr, IriString};
use serde_json::Value;

use crate::{
    context::Context,
    error::{ErrorCode, Result},
    json::to_ref_array,
    processor::Processor,
    remote::{LoadDocumentOptions, LoadRemoteDocument, Profile, RemoteDocument},
};

use self::ctx_def::process_context_definition;

mod ctx_def;

/// Optional parameters for context processing algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalParams {
    /// Remote contexts.
    remote_contexts: HashSet<IriString>,
    /// "Override protected" flag.
    override_protected: bool,
    /// "Propagate" flag.
    propagate: bool,
}

impl OptionalParams {
    /// Creates a new default `OptionalParams`.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Sets the "override protected" flag.
    pub(crate) fn override_protected(self, override_protected: bool) -> Self {
        Self {
            override_protected,
            ..self
        }
    }
}

impl Default for OptionalParams {
    fn default() -> Self {
        Self {
            remote_contexts: Default::default(),
            override_protected: false,
            propagate: true,
        }
    }
}

/// Runs context processing algorithm and returns a new context.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#context-processing-algorithm>.
///
/// This is a wrapper for modules outside this module.
pub(crate) async fn join_value<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &Context,
    local_context: &Value,
    optional: OptionalParams,
) -> Result<Context> {
    let OptionalParams {
        remote_contexts,
        override_protected,
        propagate,
    } = optional;

    join_value_impl(
        processor,
        active_context,
        local_context,
        remote_contexts,
        override_protected,
        propagate,
        &mut Default::default(),
    )
    .await
}

/// Runs context processing algorithm and returns a new context.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#context-processing-algorithm>.
///
/// This is a wrapper for recursive call.
fn join_value_impl_recursive<'a, L: LoadRemoteDocument>(
    processor: &'a Processor<L>,
    active_context: &'a Context,
    local_context: &'a Value,
    remote_contexts: HashSet<IriString>,
    override_protected: bool,
    propagate: bool,
    remote_contexts_cache: &'a mut HashMap<IriString, Arc<RemoteDocument>>,
) -> Pin<Box<dyn Future<Output = Result<Context>> + 'a + Send>> {
    Box::pin(async move {
        join_value_impl(
            processor,
            active_context,
            local_context,
            remote_contexts,
            override_protected,
            propagate,
            remote_contexts_cache,
        )
        .await
    })
}

/// Runs context processing algorithm and returns a new context.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191018/#context-processing-algorithm>.
async fn join_value_impl<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &Context,
    local_context: &Value,
    mut remote_contexts: HashSet<IriString>,
    override_protected: bool,
    propagate: bool,
    remote_contexts_cache: &mut HashMap<IriString, Arc<RemoteDocument>>,
) -> Result<Context> {
    // Step 1
    let mut result = active_context.clone();
    // Step 2
    // NOTE: Spec says as below, but I have no idea what to do if the value of the `@propagate`
    // entry is not a boolean.
    //
    // > If _local context_ is an object containing the member `@propagate`, its value MUST be
    // > boolean `true` or `false`, set _propagate_ to that value.
    let propagate = local_context
        .get("@propagate")
        .and_then(Value::as_bool)
        .unwrap_or(propagate);
    // Step 3
    if !propagate && result.has_previous_context() {
        result.previous_context = Some(Box::new(active_context.clone()));
    }
    // Step 4
    let local_context = to_ref_array(local_context);
    // Step 5
    for context in local_context {
        // Step 5.1-
        match context {
            // Step 5.1
            Value::Null => {
                // Step 5.1.1, 5.1.2
                result = process_single_null(active_context, override_protected, propagate, result)
                    .await?;
            }
            // Step 5.2
            Value::String(context) => {
                // Step 5.2.1-5.2.6
                result = process_single_string(
                    processor,
                    active_context,
                    &mut remote_contexts,
                    override_protected,
                    propagate,
                    remote_contexts_cache,
                    result,
                    context,
                )
                .await?;
                // Step 5.2.7: Continue with the next _context_.
                // No need of explicit `continue` here.
            }
            // Step 5.4-5.13
            Value::Object(context) => {
                result = process_context_definition(
                    processor,
                    active_context,
                    &mut remote_contexts,
                    propagate,
                    result,
                    context,
                )
                .await?;
            }
            // Step 5.3
            v => {
                return Err(
                    ErrorCode::InvalidLocalContext.and_source(anyhow!("local context = {:?}", v))
                )
            }
        }
    }

    // Step 6
    Ok(result)
}

/// Processes single context which is `null`.
async fn process_single_null(
    active_context: &Context,
    override_protected: bool,
    propagate: bool,
    mut result: Context,
) -> Result<Context> {
    // Step 5.1.1
    if !override_protected && active_context.has_protected_term_definition() {
        return Err(ErrorCode::InvalidContextNullification.into());
    }
    // Step 5.1.2
    // > set result to a newly-initialized _active context_, setting _previous context_
    // > in _result_ to the previous value of _result_ if propagate is `false`.
    let previous_context = std::mem::replace(&mut result, Context::new());
    if !propagate {
        result.previous_context = Some(Box::new(previous_context));
    }

    Ok(result)
}

/// Processes single context which is a string.
#[allow(clippy::too_many_arguments)] // TODO: FIXME
async fn process_single_string<L: LoadRemoteDocument>(
    processor: &Processor<L>,
    active_context: &Context,
    remote_contexts: &mut HashSet<IriString>,
    override_protected: bool,
    propagate: bool,
    remote_contexts_cache: &mut HashMap<IriString, Arc<RemoteDocument>>,
    mut result: Context,
    context: &str,
) -> Result<Context> {
    use std::collections::hash_map::Entry;

    // Step 5.2.1
    let base = match processor.base(&active_context) {
        Some(v) => v,
        None => unimplemented!("FIXME: What to do if no base IRI available?"),
    };
    let context: &IriReferenceStr = IriReferenceStr::new(context).map_err(|e| {
        ErrorCode::Uncategorized
            .and_source(e)
            .context(format!("Expected IRI reference, but got {:?}", context))
    })?;
    let context: IriString = context.resolve_against(base.to_absolute());
    // Step 5.2.2
    if !processor.is_remote_context_limit_exceeded(remote_contexts.len()) {
        return Err(ErrorCode::ContextOverflow.and_source(anyhow!(
            "Current number of remote contexts = {:?}",
            remote_contexts.len()
        )));
    }
    remote_contexts.insert(context.clone());
    // Step 5.2.3-5.2.4
    // > If _context_ was previously dereferenced, then the processor MUST NOT do a
    // > further dereference, and _context_ is set to the previously established
    // > internal representation.
    let remote_doc: Arc<RemoteDocument> = match remote_contexts_cache.entry(context.clone()) {
        // Step 5.2.3
        Entry::Occupied(entry) => entry.into_mut().clone(),
        // Step 5.2.4, 5.2.5
        Entry::Vacant(entry) => {
            let mut load_opts = LoadDocumentOptions::new();
            load_opts.set_profile(Profile::Context);
            load_opts.set_request_profile(Profile::Context);
            let doc = processor
                .loader()
                .load(&context, load_opts)
                .await
                .map_err(|e| ErrorCode::LoadingRemoteContextFailed.and_source(e))?;
            entry.insert(doc).clone()
        }
    };
    // Step 5.2.5
    let context = remote_doc.document().get("@context").ok_or_else(|| {
        ErrorCode::InvalidRemoteContext.and_source(anyhow!("doc = {:?}", remote_doc))
    })?;
    // Step 5.2.6
    result = join_value_impl_recursive(
        processor,
        &result,
        context,
        remote_contexts.clone(),
        override_protected,
        propagate,
        remote_contexts_cache,
    )
    .await?;

    Ok(result)
}
