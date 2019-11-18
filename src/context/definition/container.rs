//! Container.

use std::{convert::TryFrom, fmt, iter};

use serde_json::Value;
use thiserror::Error as ThisError;

use crate::json::Nullable;

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

impl ContainerItem {
    /// Returns an integer with distinct single bit set.
    fn single_bit(self) -> u8 {
        let shift = match self {
            Self::Graph => 0,
            Self::Id => 1,
            Self::Index => 2,
            Self::Language => 3,
            Self::List => 4,
            Self::Set => 5,
            Self::Type => 6,
        };
        1 << shift
    }

    /// Returns an iterator of `ContainerItem` enum variants.
    fn variants() -> impl Iterator<Item = Self> {
        /// List of all variants.
        const ALL_VARIANTS: [ContainerItem; 7] = [
            ContainerItem::Graph,
            ContainerItem::Id,
            ContainerItem::Index,
            ContainerItem::Language,
            ContainerItem::List,
            ContainerItem::Set,
            ContainerItem::Type,
        ];
        ALL_VARIANTS.iter().copied()
    }
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

/// `@container` value.
///
/// This type itself is a simple container and does not do any validation.
/// Loaders are responsible to do it.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Container {
    /// Items of `@container` entry.
    items: u8,
    /// Whether to use an array form even if there are only one item.
    prefer_array: bool,
}

impl Container {
    /// Creates a new empty `Container`.
    fn new() -> Self {
        Self {
            items: 0,
            prefer_array: false,
        }
    }

    /// Forces the container to be in an array form, even if it contains only single item.
    pub(crate) fn prefer_array(&mut self) {
        self.prefer_array = true;
    }

    /// Returns the item and whether the container is in an array form,
    /// if there is only single item.
    pub(crate) fn get_single_item(self) -> Option<(ContainerItem, bool)> {
        if self.len() != 1 {
            return None;
        }
        self.iter().next().map(|item| (item, self.prefer_array))
    }

    /// Checks whether the container has the given item.
    pub(crate) fn contains(self, v: ContainerItem) -> bool {
        self.items & v.single_bit() != 0
    }

    /// Returns an iterator of profiles.
    pub(crate) fn iter(self) -> impl Iterator<Item = ContainerItem> {
        ContainerItem::variants().filter(move |v| self.contains(*v))
    }

    /// Returns the number of items.
    pub(crate) fn len(self) -> usize {
        self.items.count_ones() as usize
    }
}

impl fmt::Debug for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_single_item() {
            Some((item, false)) => item.fmt(f),
            _ => f.debug_set().entries(self.iter()).finish(),
        }
    }
}

impl From<ContainerItem> for Container {
    fn from(v: ContainerItem) -> Self {
        Self {
            items: v.single_bit(),
            prefer_array: false,
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
                .map(Container::from)
                .map_err(|e| e.prepend("Unexpected string value")),
            Value::Array(arr) => arr
                .iter()
                .map(ContainerItem::try_from)
                .collect::<Result<Container, _>>()
                .map(|mut c| {
                    c.prefer_array();
                    c
                })
                .map_err(|e| e.prepend("Unexpected value in array")),
            v => Err(ContainerLoadError::new(format_args!(
                "Unexpected value {:?}",
                v
            ))),
        }
    }
}

impl iter::FromIterator<ContainerItem> for Container {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = ContainerItem>,
    {
        let mut v = Container::new();
        v.extend(iter.into_iter());
        v
    }
}

impl iter::Extend<ContainerItem> for Container {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = ContainerItem>,
    {
        iter.into_iter().for_each(|v| self.items |= v.single_bit());
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
