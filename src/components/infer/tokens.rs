use crate::app::AppState;
use anyhow::{Error, Ok, Result};
use serde_json::Value;

pub fn to_tokens(state: &AppState, data: Value) -> Result<Vec<u16>> {
    Ok(match data {
        Value::String(s) => state.tokenize(&s.into_bytes())?,
        Value::Array(v) => v
            .into_iter()
            .map(|x| match x {
                Value::String(x) => Ok(state.tokenize(&x.into_bytes()))?,
                Value::Number(x) => Ok(vec![x
                    .as_u64()
                    .ok_or(Error::msg("Token must be a u16 integer!"))?
                    as u16]),
                _ => Err(Error::msg(
                    "Can only interpret tokens from a string or an integer!",
                )),
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect(),
        _ => return Err(Error::msg("Must be a string or a list of integers!")),
    })
}

pub fn to_token_vec(state: &AppState, data: Value) -> Result<Vec<Vec<u16>>> {
    if let Value::Array(data) = data {
        data.into_iter().map(|x| to_tokens(state, x)).collect()
    } else {
        Ok(vec![to_tokens(state, data)?])
    }
}
