//! Nullable value.

use serde_json::Value;

/// Nullable JSON value.
///
/// Usually used in `Option<Nullable<T>>` form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Nullable<T> {
    /// Null.
    Null,
    /// Non-null value.
    Value(T),
}

impl<T> Nullable<T> {
    /// Creates `Nullable<&T>` from `Nullable<T>`.
    pub fn as_ref(&self) -> Nullable<&T> {
        match self {
            Nullable::Null => Nullable::Null,
            Nullable::Value(v) => Nullable::Value(v),
        }
    }

    /// Converts `Nullable<T>` into `Nullable<U>`.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Nullable<U> {
        match self {
            Nullable::Null => Nullable::Null,
            Nullable::Value(v) => Nullable::Value(f(v)),
        }
    }
}

impl<T> Default for Nullable<T> {
    fn default() -> Self {
        Nullable::Null
    }
}

impl<T> From<T> for Nullable<T> {
    fn from(v: T) -> Self {
        Nullable::Value(v)
    }
}

impl<T> Into<Option<T>> for Nullable<T> {
    fn into(self) -> Option<T> {
        match self {
            Nullable::Null => None,
            Nullable::Value(v) => Some(v),
        }
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => Nullable::Value(v),
            None => Nullable::Null,
        }
    }
}

impl<T: Into<serde_json::Value>> Into<Value> for Nullable<T> {
    fn into(self) -> Value {
        match self {
            Nullable::Null => Value::Null,
            Nullable::Value(v) => v.into(),
        }
    }
}
