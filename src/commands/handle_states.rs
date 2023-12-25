use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, components::infer::tokens::to_token_vec};

#[derive(Debug, Deserialize)]
struct StateCreate {
    id: String,
    dump_id: Option<String>,
}

#[inline]
pub async fn create_state(data: Option<Value>, state: AppState) -> Result<Value> {
    let StateCreate { id, dump_id } = serde_json::from_value(
        data.ok_or(Error::msg("Field data is needed to specify state id!"))?,
    )?;
    match dump_id {
        Some(dump_id) => state.load_state(id, dump_id).await?,
        None => state.0.states.create_state(id.as_str()).await?,
    };
    Ok(Value::Null)
}

#[derive(Debug, Deserialize)]
struct StateCopy {
    source: String,
    destination: String,
    shallow: Option<bool>,
}

#[inline]
pub async fn copy_state(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let StateCopy {
            source,
            destination,
            shallow,
        } = serde_json::from_value(data)?;
        let shallow = shallow.unwrap_or(false);
        state
            .0
            .states
            .copy_state(&source, &destination, shallow)
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg(
            "Field data is needed to specify source state and destination id!",
        ))
    }
}

#[inline]
pub async fn delete_state(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        state
            .0
            .states
            .delete_state(data.as_str().ok_or(Error::msg(
                "data should be a string representing state id you want to delete!",
            ))?)
            .await
            .map(|_| Value::Null)
    } else {
        Err(Error::msg("Field data is needed to specify state id!"))
    }
}

#[derive(Debug, Deserialize)]
struct StateUpdate {
    states: Vec<String>,
    tokens: Value,
    probs_dist: Option<Vec<u16>>,
}

#[inline]
pub async fn update_state(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let StateUpdate {
            states,
            tokens,
            probs_dist,
        } = serde_json::from_value(data)?;
        let tokens = to_token_vec(&state, tokens)?;
        state
            .update_state(states, tokens, probs_dist)
            .await
            .and_then(|v| Ok(serde_json::to_value(v)?))
    } else {
        Err(Error::msg(
            "Field data is needed to specify state id and tokens!",
        ))
    }
}

#[derive(Debug, Deserialize)]
struct StateDump {
    state_id: String,
    dump_id: String,
}

#[inline]
pub async fn dump_state(data: Option<Value>, state: AppState) -> Result<Value> {
    let StateDump { state_id, dump_id } =
        serde_json::from_value(data.ok_or(Error::msg("Field empty!"))?)?;

    state.dump_state(state_id, dump_id).await?;
    Ok(Value::Null)
}

#[inline]
pub async fn delete_dump(data: Option<Value>, state: AppState) -> Result<Value> {
    state
        .delete_dump(serde_json::from_value(
            data.ok_or(Error::msg("Field must be a string!"))?,
        )?)
        .await?;
    Ok(Value::Null)
}
