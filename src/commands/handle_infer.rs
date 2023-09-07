use anyhow::{Error, Result};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::{app::SharedState, commands::helpers};

#[derive(Debug, Deserialize)]
struct InferPayload {
    tokens: Vec<Value>,
    states: Vec<String>,
    transformers: Vec<Vec<String>>,
    sampler: String,
    update_prompt: bool,
}

fn transform_logits(
    app_state: SharedState,
    logits: Vec<f32>,
    transformers: &Vec<String>,
) -> Result<Vec<f32>> {
    let mut logits = logits;
    for transformer in transformers {
        app_state
            .transformers
            .transform_logits(transformer, &mut logits)?
    }
    Ok(logits)
}
async fn infer_and_sample(
    app_state: SharedState,
    state_ids: &Vec<String>,
    transformers: &Vec<Vec<String>>,
    tokens: Vec<Vec<u16>>,
    sampler: &String,
) -> Result<u16> {
    let logits = app_state
        .infer(state_ids.clone(), tokens)
        .await?
        .into_par_iter()
        .map(|x| x.0)
        .zip(transformers.par_iter())
        .map(|(logits, t_ids)| transform_logits(app_state.clone(), logits, t_ids))
        .collect::<Result<Vec<_>>>()?;

    let probs = app_state.model.softmax(logits)?;

    return app_state.samplers.sample_token(sampler, probs);
}

pub async fn infer(data: Option<Value>, state: SharedState) -> Result<Value> {
    if let Some(data) = data {
        let InferPayload {
            tokens,
            states,
            transformers,
            sampler,
            update_prompt,
        } = serde_json::from_value::<InferPayload>(data)?;

        if states.iter().any(|x| !state.has_state(x)) {
            return Err(Error::msg("One or more state ids not exist!"));
        }

        if transformers
            .iter()
            .flatten()
            .any(|x| !state.transformers.has_transformer(x))
        {
            return Err(Error::msg("One or more transformer ids not exist!"));
        }

        if !state.samplers.has_sampler(&sampler) {
            return Err(Error::msg("Sampler id does not exist!"));
        }

        let tokens = tokens
            .into_iter()
            .map(|v| helpers::to_tokens(&state, v))
            .collect::<Result<Vec<_>>>()?;

        if update_prompt {
            transformers
                .par_iter()
                .zip(tokens.par_iter())
                .for_each(|(t_ids, tokens)| {
                    for t_id in t_ids {
                        let _ = state.transformers.update_transformer(t_id, tokens);
                    }
                });

            let _ = state.samplers.update_sampler(&sampler, &tokens);
        }

        let result = {
            let mut out_tokens = Vec::with_capacity(4);
            // Feed prompt first
            out_tokens.push(
                infer_and_sample(state.clone(), &states, &transformers, tokens, &sampler).await?,
            );

            loop {
                if let Ok(Ok(result)) = state
                    .tokenizer
                    .decode(&out_tokens.as_slice())
                    .map(|x| String::from_utf8(x))
                {
                    break result;
                }

                // Not ready, infer next one using last token
                out_tokens.push(
                    infer_and_sample(
                        state.clone(),
                        &states,
                        &transformers,
                        vec![vec![*out_tokens.last().unwrap(); states.len()]],
                        &sampler,
                    )
                    .await?,
                );
            }
        };

        Ok(Value::String(result))
    } else {
        Err(Error::msg(
            "Field data is needed to specify infer pipeline!",
        ))
    }
}
