use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::SharedState;

use super::helpers;

#[derive(Debug, Deserialize)]
struct SamplerArgs {
    type_id: String,
    params: Option<Value>,
}

#[inline]
pub async fn create_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
        let SamplerArgs { type_id, params } = serde_json::from_value(data)?;
        state
            .samplers
            .create_sampler(type_id, state.clone(), params)
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify sampler type_id and params!",
        ))
    }
}

#[derive(Debug, Deserialize)]
struct SamplerCopy {
    source: String,
    destination: String,
}

#[inline]
pub async fn copy_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
        let SamplerCopy {
            source,
            destination,
        } = serde_json::from_value(data)?;
        state
            .samplers
            .copy_sampler(source, destination)
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify source sampler and destination id!",
        ))
    }
}

#[inline]
pub async fn delete_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
        state
            .samplers
            .delete_sampler(
                data.as_str()
                    .ok_or(Error::msg(
                        "data should be a string representing sampler id you want to delete!",
                    ))?
                    .to_string(),
            )
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify sampler id!"))
    }
}

#[derive(Debug, Deserialize)]
struct SamplerUpdate {
    id: String,
    tokens: Value,
}

#[inline]
pub async fn update_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
        let SamplerUpdate { id, tokens } = serde_json::from_value(data)?;
        let tokens = helpers::to_tokens(&state, tokens).await?;
        state
            .samplers
            .update_sampler(id, tokens)
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify transformer id and tokens!",
        ))
    }
}

#[inline]
pub async fn reset_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
        state
            .samplers
            .reset_sampler(
                data.as_str()
                    .ok_or(Error::msg(
                        "data should be a string representing sampler id you want to reset!",
                    ))?
                    .to_string(),
            )
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify sampler id!"))
    }
}
