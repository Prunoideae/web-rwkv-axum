use crate::app::AppState;
use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

pub async fn create_pipeline(data: Option<Value>, state: AppState) -> Result<Value> {
    state
        .0
        .pipelines
        .create_pipeline(&state, data.ok_or(Error::msg("Payload required"))?)?;
    Ok(Value::Null)
}

pub async fn copy_pipeline(data: Option<Value>, state: AppState) -> Result<Value> {
    #[derive(Deserialize)]
    struct Copy {
        source: String,
        destination: String,
    }
    let Copy {
        source,
        destination,
    } = serde_json::from_value::<Copy>(data.ok_or(Error::msg("Payload required"))?)?;
    state.0.pipelines.copy_pipeline(&source, destination).await?;
    Ok(Value::Null)
}

pub async fn delete_pipeline(data: Option<Value>, state: AppState) -> Result<Value> {
    state
        .0
        .pipelines
        .remove_pipeline(&serde_json::from_value::<String>(
            data.ok_or(Error::msg("Payload required"))?,
        )?)?;
    Ok(Value::Null)
}

pub async fn reset_pipeline(data: Option<Value>, state: AppState) -> Result<Value> {
    state
        .0
        .pipelines
        .get_pipeline(&serde_json::from_value::<String>(
            data.ok_or(Error::msg("Payload required"))?,
        )?)?
        .lock()
        .await
        .reset_all();
    Ok(Value::Null)
}
