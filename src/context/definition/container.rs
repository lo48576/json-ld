//! Container.

use std::convert::TryFrom;

use serde_json::Value;
use thiserror::Error as ThisError;

use crate::json::Nullable;

/// `@container` value.
///
/// This type itself is a simple container and does not do any validation.
/// Loaders are responsible to do it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Container {
    /// Single item.
    Single(ContainerItem),
    /// Array of items.
    Array(Vec<ContainerItem>),
}

impl Container {
    /// Checks whether the container has the given item.
    pub(crate) fn contains(&self, v: ContainerItem) -> bool {
        match self {
            Self::Single(item) => *item == v,
            Self::Array(arr) => arr.contains(&v),
        }
    }

    /// Returns an iterator of the container.
    pub(crate) fn iter(&self) -> impl Iterator<Item = ContainerItem> + '_ {
        match self {
            Self::Single(s) => Some(*s).into_iter().chain((&[]).iter().copied()),
            Self::Array(arr) => None.into_iter().chain(arr.iter().copied()),
        }
    }

    /// Returns the number of container items.
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Array(arr) => arr.len(),
        }
    }
}

impl TryFrom<&Value> for Nullable<Container> {
    type Error = ContainerLoadError;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Null => Ok(Nullable::Null),
            v => Container::try_from(v).map(Nullable::Value),
        }
    }
}

impl TryFrom<&Value> for Container {
    type Error = ContainerLoadError;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::String(s) => s
                .parse::<ContainerItem>()
                .map(Container::Single)
                .map_err(|e| e.prepend("Unexpected string value")),
            Value::Array(arr) => arr
                .iter()
                .map(ContainerItem::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(Container::Array)
                .map_err(|e| e.prepend("Unexpected value in array")),
            v => Err(ContainerLoadError::new(format_args!(
                "Unexpected value {:?}",
                v
            ))),
        }
    }
}

/// Possible items for `@container`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerItem {
    /// `@graph`.
    Graph,
    /// `@id`.
    Id,
    /// `@index`.
    Index,
    /// `@language`.
    Language,
    /// `@list`.
    List,
    /// `@set`.
    Set,
    /// `@type`.
    Type,
}

impl TryFrom<&Value> for ContainerItem {
    type Error = ContainerLoadError;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::String(s) => s
                .parse::<ContainerItem>()
                .map_err(|e| e.prepend("Unexpected string in array")),
            v => Err(ContainerLoadError::new(format_args!(
                "Unexpected value {:?} in array",
                v
            ))),
        }
    }
}

impl TryFrom<&str> for ContainerItem {
    type Error = ContainerLoadError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "@graph" => Ok(Self::Graph),
            "@id" => Ok(Self::Id),
            "@index" => Ok(Self::Index),
            "@language" => Ok(Self::Language),
            "@list" => Ok(Self::List),
            "@set" => Ok(Self::Set),
            "@type" => Ok(Self::Type),
            v => Err(ContainerLoadError::new(format_args!(
                "Unknown item: {:?}",
                v
            ))),
        }
    }
}

impl std::str::FromStr for ContainerItem {
    type Err = ContainerLoadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::try_from(s)
    }
}

/// Container load error.
#[derive(Debug, Clone, ThisError)]
#[error("Failed to load `@container`: {msg}")]
pub struct ContainerLoadError {
    /// Message.
    msg: String,
}

impl ContainerLoadError {
    /// Creates a new error.
    fn new(msg: impl std::fmt::Display) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }

    /// Prepends the given message.
    fn prepend(self, msg: impl std::fmt::Display) -> Self {
        Self {
            msg: format!("{}: {}", msg, self.msg),
        }
    }
}
