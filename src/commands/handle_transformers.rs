use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::helpers;

#[derive(Debug, Deserialize)]
struct TransformerArgs {
    type_id: String,
    params: Option<Value>,
}

#[inline]
pub async fn create_transformer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let TransformerArgs { type_id, params } = serde_json::from_value(data)?;
        state
            .0
            .transformers
            .create_transformer(type_id, state.clone(), params)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify transformer type_id and params!",
        ))
    }
}

#[derive(Debug, Deserialize)]
struct TransformerCopy {
    source: String,
    destination: String,
}

#[inline]
pub async fn copy_transformer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let TransformerCopy {
            source,
            destination,
        } = serde_json::from_value(data)?;
        state
            .0
            .transformers
            .copy_transformer(source, destination)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify source transformer and destination id!",
        ))
    }
}

#[inline]
pub async fn delete_transformer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .transformers
            .delete_transformer(
                data.as_str()
                    .ok_or(Error::msg(
                        "data should be a string representing transformer id you want to delete!",
                    ))?
                    .to_string(),
            )
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify transformer id!",
        ))
    }
}

#[derive(Debug, Deserialize)]
struct TransformerUpdate {
    id: String,
    tokens: Value,
}

#[inline]
pub async fn update_transformer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let TransformerUpdate { id, tokens } = serde_json::from_value(data)?;
        let tokens = helpers::to_tokens(&state, tokens)?;
        state
            .0
            .transformers
            .update_transformer(&id, &tokens)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify transformer id and tokens!",
        ))
    }
}

#[inline]
pub async fn reset_transformer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .transformers
            .reset_transformer(
                data.as_str()
                    .ok_or(Error::msg(
                        "data should be a string representing transformer id you want to reset!",
                    ))?
                    .to_string(),
            )
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify transformer id!",
        ))
    }
}
