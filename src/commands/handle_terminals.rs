use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

#[derive(Debug, Deserialize)]
struct TerminalArgs {
    id: String,
    data: Value,
}

#[inline]
pub async fn create_terminal(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let TerminalArgs { id, data } = serde_json::from_value(data)?;
        state
            .0
            .terminals
            .create_terminal(id, state.clone(), data)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify terminal type_id and params!",
        ))
    }
}

#[derive(Debug, Deserialize)]
struct TerminalCopy {
    source: String,
    destination: String,
}

#[inline]
pub async fn copy_terminal(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let TerminalCopy {
            source,
            destination,
        } = serde_json::from_value(data)?;
        state
            .0
            .terminals
            .copy_terminal(source, destination)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify source terminal and destination id!",
        ))
    }
}

#[inline]
pub async fn delete_terminal(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .terminals
            .delete_terminal(data.as_str().ok_or(Error::msg(
                "data should be a string representing terminal id you want to delete!",
            ))?)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify terminal id!"))
    }
}

#[inline]
pub async fn reset_terminal(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .terminals
            .reset_terminal(data.as_str().ok_or(Error::msg(
                "data should be a string representing terminal id you want to reset!",
            ))?)
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify terminal id!"))
    }
}
