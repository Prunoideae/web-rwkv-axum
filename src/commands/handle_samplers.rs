use anyhow::{Error, Result};
use serde_json::Value;

use crate::app::SharedState;

#[inline]
pub async fn create_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}

#[inline]
pub async fn copy_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}

#[inline]
pub async fn delete_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}

#[inline]
pub async fn reset_sampler(data: Option<Value>, state: SharedState) -> Result<Value> {
    todo!()
}
