//! JSON helpers.

use serde_json::{Map as JsonMap, Value};

pub(crate) use self::nullable::Nullable;

mod nullable;

/// Returns a map with single key-value entry.
pub(crate) fn single_entry_map(
    id: impl Into<String>,
    value: impl Into<Value>,
) -> JsonMap<String, Value> {
    let mut map = JsonMap::new();
    map.insert(id.into(), value.into());
    map
}

/// Converts the given JSON value to a slice of elements.
pub(crate) fn to_ref_array(v: &Value) -> &[Value] {
    match v {
        Value::Array(v) => v,
        v => std::slice::from_ref(v),
    }
}
