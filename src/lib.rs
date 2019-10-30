//! JSON-LD processing library.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::error::{Error, ErrorCode, Result};

mod error;
