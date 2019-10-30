//! JSON-LD processing library.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::{
    context::Context,
    error::{Error, ErrorCode, Result},
    processor::ProcessorOptions,
};

pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod expand;
pub(crate) mod iri;
pub(crate) mod json;
pub(crate) mod processor;
