use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::SharedState;

#[inline]
pub async fn create_state(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
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
    } else {
        Err(Error::msg("Field data is needed to specify state id!"))
    }
}

#[derive(Debug, Deserialize)]
struct StateCopy {
    source: String,
    destination: String,
}

#[inline]
pub async fn copy_state(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
        let StateCopy {
            source,
            destination,
        } = serde_json::from_value(data)?;
        state
            .copy_state(source, destination)
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify source state and destination id!",
        ))
    }
}

#[inline]
pub async fn delete_state(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
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
    } else {
        Err(Error::msg("Field data is needed to specify state id!"))
    }
}
