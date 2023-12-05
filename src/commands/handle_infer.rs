use std::time::Duration;

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::timeout;

use crate::{
    app::AppState,
    components::infer::{
        tokens::to_tokens,
        updates::{ResetSetting, UpdateSetting},
    },
};

pub async fn infer(data: Option<Value>, state: AppState) -> Result<Value> {
    #[derive(Debug, Deserialize)]
    struct InferPayload {
        tokens: Vec<Value>,
        states: Vec<String>,
        pipeline: String,
        update_prompt: Option<Value>,
        reset_on_exhaustion: Option<Value>,
        timeout: Option<usize>,
    }

    #[derive(Debug, Serialize)]
    struct InferResponse {
        prompt_tokens: usize,
        inferred_tokens: usize,
        result: String,
        last_token: u16,
        end_reason: &'static str,
    }

    let InferPayload {
        tokens,
        states,
        pipeline,
        update_prompt,
        reset_on_exhaustion,
        timeout: timeout_millis,
    } = serde_json::from_value(data.ok_or(Error::msg("Payload required!"))?)?;

    let tokens = tokens
        .into_iter()
        .map(|x| to_tokens(&state, x))
        .collect::<Result<Vec<_>>>()?;

    let prompt_tokens = tokens.iter().fold(0, |x, y| x + y.len());

    let ticket = timeout(
        Duration::from_millis(timeout_millis.unwrap_or(20 * 1000) as u64),
        state.0.states.create_ticket(states),
    )
    .await??;

    let pipeline = state.0.pipelines.get_pipeline(&pipeline)?;

    let mut lock = pipeline.lock().await;
    let update_setting = UpdateSetting::from_value(&lock.get_transformer_shape(), update_prompt)?;
    let reset_setting =
        ResetSetting::from_value(&lock.get_transformer_shape(), reset_on_exhaustion)?;

    let (last_token, inferred_tokens, end_reason) = {
        lock.infer(
            ticket,
            reset_setting,
            update_setting,
            tokens,
            state.0.config.model.get_max_infer_tokens(),
            &state,
        )
    }
    .await?;

    drop(lock);

    Ok(serde_json::to_value(InferResponse {
        prompt_tokens,
        inferred_tokens: inferred_tokens.len(),
        result: String::from_utf8_lossy(&state.0.tokenizer.decode(&inferred_tokens)?).to_string(),
        last_token,
        end_reason,
    })?)
}
