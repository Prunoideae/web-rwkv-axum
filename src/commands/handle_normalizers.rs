use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

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
