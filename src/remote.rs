//! Remote-document related stuff.

use async_trait::async_trait;
use serde_json::Value;

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
    async fn load(&self, iri: &str) -> Result<RemoteDocument, Self::Error>;
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
