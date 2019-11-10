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
