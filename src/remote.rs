//! Remote-document related stuff.

use std::sync::Arc;

use async_trait::async_trait;
use iri_string::types::IriStr;
use serde_json::Value;

pub use self::profile::{Profile, RequestProfile};

mod profile;

/// A trait for types which can be used as remote document loader.
///
/// NOTE: This trait uses `async_trait` crate to make trait method async fn.
/// You should specify `#[async_trait]` for trait impl block if you implement this trait for your
/// custom loader type.
/// `async_trait` trait is re-exported by this (json-ld) crate, so you can do
/// `use json_ld::async_trait::async_trait`.
#[async_trait]
pub trait LoadRemoteDocument: Send + Sync {
    /// Error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Loads a remote context.
    ///
    /// This returns `Arc` to make query result cacheable at low cost.
    ///
    /// JSON-LD spec requires processors to cache query result inside algorithm (but not globally).
    /// Algorithms provided by this crate internally caches the result returned from `load()`, so
    /// implementors of this trait does not need to think about caching.
    ///
    /// Implementors of this trait can use any caching strategy.
    /// For example they can do the below safely:
    ///
    /// * Do network operations every time this method is called.
    /// * Cache result for every document loaders, but don't share the caches among other loaders.
    /// * Cache the data globally, and share caches among all loaders.
    async fn load(
        &self,
        iri: &IriStr,
        options: LoadDocumentOptions,
    ) -> Result<Arc<RemoteDocument>, Self::Error>;
}

/// Options for `LoadRemoteDocument::load()`.
///
/// See <https://www.w3.org/TR/2019/WD-json-ld11-api-20191112/#loaddocumentoptions>.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadDocumentOptions {
    /// A flag to let the loader extract JSON-LD script elements in HTML, if necessary.
    ///
    /// > If set to `true`, when extracting JSON-LD script elements from HTML, unless a specific
    /// > fragment identifier is targeted, extracts all encountered JSON-LD script elements using an
    /// > array form, if necessary.
    extract_all_scripts: bool,
    /// Default fallback profile.
    ///
    /// > When the resulting `contentType` is `text/html`, this option determines the profile to use
    /// > for selecting a JSON-LD script elements.
    profile: Option<Profile>,
    /// One or more profiles to use in the request as a `profile` parameter.
    ///
    /// > One or more IRIs to use in the request as a `profile` parameter. (See IANA Considerations
    /// in \[JSON-LD11\]).
    request_profile: RequestProfile,
}

impl LoadDocumentOptions {
    /// Creates a new `LoadDocumentOptions`.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Sets the given profile.
    pub(crate) fn set_profile(&mut self, profile: impl Into<Option<Profile>>) {
        self.profile = profile.into();
    }

    /// Sets the given request profile.
    pub(crate) fn set_request_profile(&mut self, request_profile: impl Into<RequestProfile>) {
        self.request_profile = request_profile.into();
    }

    /// Returns whether the loader should extract JSON-LD script elements in HTML, if necessary.
    ///
    /// > If set to `true`, when extracting JSON-LD script elements from HTML, unless a specific
    /// > fragment identifier is targeted, extracts all encountered JSON-LD script elements using an
    /// > array form, if necessary.
    pub fn should_extract_all_scripts(&self) -> bool {
        self.extract_all_scripts
    }

    /// Returns default fallback profile of the document.
    ///
    /// > When the resulting `contentType` is `text/html`, this option determines the profile to use
    /// > for selecting a JSON-LD script elements.
    pub fn profile(&self) -> Option<Profile> {
        self.profile
    }

    /// Returns profiles to use in the request as a `profile` parameter.
    ///
    /// > One or more IRIs to use in the request as a `profile` parameter. (See IANA Considerations
    /// in \[JSON-LD11\]).
    pub fn request_profile(&self) -> RequestProfile {
        self.request_profile
    }
}

/// Remote document.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteDocument {
    /// Context URL.
    context_url: Option<String>,
    /// Document IRI.
    document_url: String,
    /// Document.
    document: Value,
}

impl RemoteDocument {
    /// Returns a reference to the document.
    pub fn document(&self) -> &Value {
        &self.document
    }

    /// Returns the document with ownership.
    pub fn into_document(self) -> Value {
        self.document
    }
}
