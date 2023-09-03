use anyhow::{Error, Result};
use serde_json::Value;

use crate::app::SharedState;

#[inline]
pub async fn create_transformer(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}

#[inline]
pub async fn copy_transformer(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}

#[inline]
pub async fn delete_transformer(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}

#[inline]
pub async fn update_transformer(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}

#[inline]
pub async fn reset_transformer(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}
