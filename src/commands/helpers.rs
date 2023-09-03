use crate::app::SharedState;
use anyhow::{Error, Ok, Result};
use serde_json::Value;

pub async fn to_tokens(state: &SharedState, data: Value) -> Result<Vec<u16>> {
    Ok(match data {
        Value::String(s) => state.tokenize(&s.into_bytes()).await?,
        Value::Array(v) => serde_json::from_value(Value::Array(v))?,
        _ => return Err(Error::msg("Must be a string or a list of integers!")),
    })
}
