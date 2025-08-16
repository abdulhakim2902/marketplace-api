use anyhow::Context;
use serde::Serialize;
use serde_json::{Map, Value};

pub fn to_map<T: Serialize>(data: &T) -> anyhow::Result<Option<Map<String, Value>>> {
    let json_string = serde_json::to_string(data).context("Failed to stringify struct")?;
    let value: Value = serde_json::from_str(&json_string).context("Failed to convert to value")?;

    let struct_map = value.as_object().cloned();

    Ok(struct_map)
}
