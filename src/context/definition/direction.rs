//! Direction.

use std::convert::TryFrom;

use serde_json::Value;
use thiserror::Error as ThisError;

use crate::json::Nullable;

/// Direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// `ltr`.
    Ltr,
    /// `rtl`.
    Rtl,
}

impl TryFrom<&Value> for Nullable<Direction> {
    type Error = DirectionLoadError;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Null => Ok(Nullable::Null),
            Value::String(s) => s.parse().map(Nullable::Value),
            v => Err(DirectionLoadError::new(format_args!(
                "Expected string as `@direction`, but got {:?}",
                v
            ))),
        }
    }
}

impl TryFrom<&str> for Direction {
    type Error = DirectionLoadError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "ltr" => Ok(Direction::Ltr),
            "rtl" => Ok(Direction::Rtl),
            v => Err(DirectionLoadError::new(format_args!(
                "Invalid direction {:?}",
                v
            ))),
        }
    }
}

impl std::str::FromStr for Direction {
    type Err = DirectionLoadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::try_from(s)
    }
}

/// Direction load error.
#[derive(Debug, Clone, ThisError)]
#[error("Failed to load `@direction`: {msg}")]
pub struct DirectionLoadError {
    /// Message.
    msg: String,
}

impl DirectionLoadError {
    /// Creates a new error.
    fn new(msg: impl std::fmt::Display) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}
