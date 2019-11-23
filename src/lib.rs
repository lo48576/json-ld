//! JSON-LD processing library.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

// Re-export for use with third-party implementation of `LoadRemoteDocument` trait.
pub use async_trait;
pub use iri_string;

pub use self::{
    context::Context,
    error::{Error, ErrorCode, Result},
    processor::{Processor, ProcessorOptions},
    remote::{LoadRemoteDocument, RemoteDocument},
};

pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod expand;
pub(crate) mod iri;
pub(crate) mod json;
pub(crate) mod processor;
pub(crate) mod remote;
