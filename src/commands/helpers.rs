use crate::app::SharedState;
use anyhow::{Error, Ok, Result};
use serde_json::Value;

pub fn to_tokens(state: &SharedState, data: Value) -> Result<Vec<u16>> {
    Ok(match data {
        Value::String(s) => state.tokenize(&s.into_bytes())?,
        Value::Array(v) => serde_json::from_value(Value::Array(v))?,
        _ => return Err(Error::msg("Must be a string or a list of integers!")),
    })
}

pub fn to_token_vec(state: &SharedState, data: Value) -> Result<Vec<Vec<u16>>> {
    if let Value::Array(data) = data {
        data.into_iter().map(|x| to_tokens(state, x)).collect()
    } else {
        Ok(vec![to_tokens(state, data)?])
    }
}
