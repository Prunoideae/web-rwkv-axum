use std::time::Duration;

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::timeout;

use crate::{
    app::AppState,
    components::{
        infer::{
            reset::ResetSetting, state::state_flag_from_value, tokens::to_tokens, SamplePipeline,
        },
        InferenceInterruption,
    },
};

#[derive(Debug, Deserialize)]
struct InferPayload {
    tokens: Vec<Value>,
    states: Vec<String>,
    transformers: Vec<Vec<String>>,
    sampler: String,
    normalizer: Option<String>,
    terminal: String,
    update_prompt: Option<bool>,
    update_states: Option<Value>,
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

pub async fn infer(data: Option<Value>, state: AppState) -> Result<Value> {
    let InferPayload {
        tokens,
        states,
        transformers,
        sampler,
        terminal,
        normalizer,
        update_prompt,
        update_states,
        reset_on_exhaustion,
        timeout: timeout_millis,
    } = serde_json::from_value(data.ok_or(Error::msg("Payload required!"))?)?;

    let max_tokens = state.0.config.model.get_max_infer_tokens();

    let update_prompt = update_prompt.unwrap_or(true);
    let states_size = states.len();
    let states_flag = state_flag_from_value(&states, update_states)?;

    let tokens = tokens
        .into_iter()
        .map(|tokens| to_tokens(&state, tokens))
        .collect::<Result<Vec<_>>>()?;
    let prompt_tokens = tokens.iter().fold(0, |x, y| x + y.len());

    let mut ticket = timeout(
        Duration::from_millis(timeout_millis.unwrap_or(1000 * 20) as u64),
        state.0.states.create_ticket(states, states_flag),
    )
    .await??;

    let mut sample_pipeline = SamplePipeline::new(
        &state,
        &transformers,
        &sampler,
        &terminal,
        &normalizer,
        ResetSetting::from_value(&transformers, reset_on_exhaustion)?,
    )?;

    if update_prompt {
        sample_pipeline.update(&tokens).map_err(|e| match e {
            InferenceInterruption::Error(e) => e,
            InferenceInterruption::Exhaustion => {
                Error::msg("Pipeline is exhausted before starting.")
            }
        })?;
    }

    let logits = ticket.infer(tokens).await;
    let mut last_token = tokio::task::block_in_place(|| sample_pipeline.sample(logits, &state));
    let mut inferred_tokens = vec![last_token];

    let end_reason = loop {
        if sample_pipeline.terminate(&inferred_tokens, inferred_tokens.len())? {
            break "by_terminal";
        }
        if last_token == 0 {
            break "by_eos";
        }
        if inferred_tokens.len() >= max_tokens {
            break "by_max_tokens";
        }
        let token_vec = vec![vec![last_token]; states_size];
        match sample_pipeline.update(&token_vec) {
            Ok(_) => (),
            Err(InferenceInterruption::Exhaustion) => {
                sample_pipeline.reset();
                break "by_exhaustion";
            }
            Err(InferenceInterruption::Error(e)) => Err(e)?,
        }

        let logits = ticket.infer(token_vec).await;
        last_token = tokio::task::block_in_place(|| sample_pipeline.sample(logits, &state));
        inferred_tokens.push(last_token);
    };

    Ok(serde_json::to_value(InferResponse {
        prompt_tokens,
        inferred_tokens: inferred_tokens.len(),
        result: String::from_utf8_lossy(&state.0.tokenizer.decode(&inferred_tokens)?).to_string(),
        last_token,
        end_reason,
    })?)
}
