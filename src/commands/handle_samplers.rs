use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    app::AppState,
    components::{infer::tokens::to_token_vec, InferenceInterruption},
};

#[derive(Debug, Deserialize)]
struct SamplerArgs {
    id: String,
    data: Value,
}

#[inline]
pub async fn create_sampler(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let SamplerArgs { id, data } = serde_json::from_value(data)?;
        state
            .0
            .samplers
            .create_sampler(id, state.clone(), data)
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
pub async fn copy_sampler(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let SamplerCopy {
            source,
            destination,
        } = serde_json::from_value(data)?;
        state
            .0
            .samplers
            .copy_sampler(source, destination)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify source sampler and destination id!",
        ))
    }
}

#[inline]
pub async fn delete_sampler(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .samplers
            .delete_sampler(data.as_str().ok_or(Error::msg(
                "data should be a string representing sampler id you want to delete!",
            ))?)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify sampler id!"))
    }
}

#[derive(Debug, Deserialize)]
struct SamplerUpdate {
    sampler: String,
    tokens: Value,
}

#[inline]
pub async fn update_sampler(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let SamplerUpdate { sampler, tokens } = serde_json::from_value(data)?;
        let tokens = to_token_vec(&state, tokens)?;
        state
            .0
            .samplers
            .update_sampler(&sampler, &tokens)
            .map_err(|interruption| match interruption {
                InferenceInterruption::Exhaustion => Error::msg("Sampler is exhausted!"),
                InferenceInterruption::Error(e) => e,
            })
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify transformer id and tokens!",
        ))
    }
}

#[inline]
pub async fn reset_sampler(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .samplers
            .reset_sampler(data.as_str().ok_or(Error::msg(
                "data should be a string representing sampler id you want to reset!",
            ))?)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify sampler id!"))
    }
}
