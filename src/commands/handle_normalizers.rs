use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, components::InferenceInterruption};

use super::helpers;

#[derive(Debug, Deserialize)]
struct NormalizerArgs {
    id: String,
    data: Value,
}

#[inline]
pub async fn create_normalizer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let NormalizerArgs { id, data } = serde_json::from_value(data)?;
        state
            .0
            .normalizers
            .create_normalizer(id, state.clone(), data)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify normalizer type_id and params!",
        ))
    }
}

#[derive(Debug, Deserialize)]
struct NormalizerCopy {
    source: String,
    destination: String,
}

#[inline]
pub async fn copy_normalizer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let NormalizerCopy {
            source,
            destination,
        } = serde_json::from_value(data)?;
        state
            .0
            .normalizers
            .copy_normalizer(source, destination)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify source normalizer and destination id!",
        ))
    }
}

#[inline]
pub async fn delete_normalizer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .normalizers
            .delete_normalizer(data.as_str().ok_or(Error::msg(
                "data should be a string representing normalizer id you want to delete!",
            ))?)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify normalizer id!"))
    }
}

#[derive(Debug, Deserialize)]
struct NormalizerUpdate {
    normalizer: String,
    tokens: Value,
}

#[inline]
pub async fn update_normalizer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let NormalizerUpdate { normalizer, tokens } = serde_json::from_value(data)?;
        let tokens = helpers::to_token_vec(&state, tokens)?;
        state
            .0
            .normalizers
            .update_normalizer(&normalizer, &tokens)
            .map_err(|interruption| match interruption {
                InferenceInterruption::Exhaustion => Error::msg("Normalizer is exhausted!"),
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
pub async fn reset_normalizer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .normalizers
            .reset_normalizer(data.as_str().ok_or(Error::msg(
                "data should be a string representing normalizer id you want to reset!",
            ))?)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify normalizer id!"))
    }
}
