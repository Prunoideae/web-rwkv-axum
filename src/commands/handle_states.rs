use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::SharedState;

pub async fn create_state(data: Value, state: SharedState) -> Result<Value> {
    state
        .create_state(
            data.as_str()
                .ok_or(Error::msg(
                    "data should be a string representing state id you want to create!",
                ))?
                .to_string(),
        )
        .await
        .map(|_| Value::Null)
}

#[derive(Debug, Deserialize)]
struct StateCopy {
    source: String,
    destination: String,
}

pub async fn copy_state(data: Value, state: SharedState) -> Result<Value> {
    let StateCopy {
        source,
        destination,
    } = serde_json::from_value(data)?;
    state
        .copy_state(source, destination)
        .await
        .map(|_| Value::Null)
}

pub async fn delete_state(data: Value, state: SharedState) -> Result<Value> {
    state
        .delete_state(
            data.as_str()
                .ok_or(Error::msg(
                    "data should be a string representing state id you want to delete!",
                ))?
                .to_string(),
        )
        .await
        .map(|_| Value::Null)
}
