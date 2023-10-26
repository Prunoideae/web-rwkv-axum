use anyhow::{Error, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{app::AppState, commands::helpers, components::InferenceInterruption};

use super::helpers::ResetSetting;

#[derive(Debug, Deserialize)]
struct InferPayload {
    tokens: Vec<Value>,
    states: Vec<String>,
    transformers: Vec<Vec<String>>,
    sampler: String,
    normalizer: Option<String>,
    terminal: String,
    update_prompt: bool,
    reset_on_exhaustion: Value,
}

fn transform_logits(
    app_state: AppState,
    mut logits: Vec<f32>,
    transformers: &Vec<String>,
) -> Result<Vec<f32>> {
    for transformer in transformers {
        logits = app_state
            .0
            .transformers
            .transform_logits(transformer, logits)?
    }
    Ok(logits)
}

async fn infer_and_sample(
    app_state: AppState,
    state_ids: &Vec<String>,
    transformers: &Vec<Vec<String>>,
    tokens: Vec<Vec<u16>>,
    sampler: &String,
    normalizer: &Option<String>,
    update_prompts: bool,
    reset_on_exhaustion: Option<&ResetSetting>,
) -> Result<u16, InferenceInterruption> {
    if update_prompts {
        tokio::task::block_in_place(|| -> Result<(), InferenceInterruption> {
            // This is the last place anything can stop the infer, if you want
            // to stop the infer in case of additional termination from
            // transformer/sampler, you must do it from updates, or the state
            // will be polluted by the token input.

            // Transformer and sampler should be aware of the exhaustion, where
            // it should know it will fail no matter what logits/probs are
            // given at sample/transformation time. and if it knows, it must
            // throw an error.
            let transformer_update = transformers
                .par_iter()
                .zip(tokens.par_iter())
                .map(|(t_ids, tokens)| {
                    for t_id in t_ids {
                        app_state.0.transformers.update_transformer(t_id, tokens)?;
                    }
                    Ok(())
                })
                .collect::<Result<Vec<()>, InferenceInterruption>>();
            let sampler_update = app_state.0.samplers.update_sampler(&sampler, &tokens);
            let normalizer_update = if let Some(normalizer) = normalizer {
                app_state
                    .0
                    .normalizers
                    .update_normalizer(&normalizer, &tokens)
            } else {
                Ok(())
            };
            let result = transformer_update
                .and(sampler_update)
                .and(normalizer_update);
            // If any interruption occurred, reset things used as it is terminated.
            // Some transformer, sampler or normalizer can be ignored.
            if let Err(InferenceInterruption::Exhaustion) = &result {
                if let Some(ResetSetting {
                    transformers: transformer_flags,
                    sampler: sampler_flag,
                    normalizer: normalizer_flag,
                }) = reset_on_exhaustion
                {
                    transformers
                        .iter()
                        .flatten()
                        .zip(transformer_flags.iter())
                        .par_bridge()
                        .for_each(|(t_id, update)| {
                            if *update {
                                app_state.0.transformers.reset_transformer(&t_id).unwrap()
                            }
                        });
                    if *sampler_flag {
                        app_state.0.samplers.reset_sampler(&sampler).unwrap();
                    }
                    if let Some(n_id) = normalizer {
                        if *normalizer_flag {
                            app_state.0.normalizers.reset_normalizer(&n_id).unwrap();
                        }
                    }
                }
            }
            result
        })?;
    }

    let logits = app_state
        .infer(state_ids.clone(), tokens)
        .await
        .map_err(|e| InferenceInterruption::Error(e))?;

    // In case if transformation is needed, we block the current thread and use rayon to
    // transform each logits
    let logits = if transformers.iter().any(|x| !x.is_empty()) {
        tokio::task::block_in_place(|| {
            logits
                .into_par_iter()
                .zip(transformers.par_iter())
                .map(|(logits, t_ids)| transform_logits(app_state.clone(), logits, t_ids))
                .collect::<Result<Vec<_>>>()
        })
        .map_err(|e| InferenceInterruption::Error(e))?
    } else {
        logits
    };
    let probs = if let Some(normalizer) = normalizer {
        app_state
            .0
            .normalizers
            .normalize_logits(normalizer, logits)
            .map_err(|e| InferenceInterruption::Error(e))?
    } else {
        app_state.softmax(logits).await
    };
    return tokio::task::block_in_place(move || app_state.0.samplers.sample_token(&sampler, probs))
        .map_err(|e| InferenceInterruption::Error(e));
}

#[derive(Debug, Serialize)]
struct InferResponse {
    value: String,
    last_token: u16,
    inferred_tokens: usize,
    end_reason: &'static str,
}

pub async fn infer(data: Option<Value>, state: AppState) -> Result<Value> {
    if let Some(data) = data {
        let InferPayload {
            tokens,
            states,
            transformers,
            sampler,
            normalizer,
            terminal,
            update_prompt,
            reset_on_exhaustion,
        } = serde_json::from_value::<InferPayload>(data)?;

        let reset_on_exhaustion = ResetSetting::from_value(&transformers, reset_on_exhaustion)?;

        if tokens.len() != states.len() || states.len() != transformers.len() {
            return Err(Error::msg(
                "State, token, transformer length must be matched!",
            ));
        }

        if states.iter().any(|x| !state.0.states.has_state(x)) {
            return Err(Error::msg("One or more state ids not exist!"));
        }

        if transformers
            .iter()
            .flatten()
            .any(|x| !state.0.transformers.has_transformer(x))
        {
            return Err(Error::msg("One or more transformer ids not exist!"));
        }

        if let Some(normalizer) = &normalizer {
            if !state.0.normalizers.has_normalizer(&normalizer) {
                return Err(Error::msg("Normalizer id doesn't exist!"));
            }
        }

        if !state.0.samplers.has_sampler(&sampler) {
            return Err(Error::msg("Sampler id does not exist!"));
        }

        let tokens = tokens
            .into_iter()
            .map(|v| helpers::to_tokens(&state, v))
            .collect::<Result<Vec<_>>>()?;

        if tokens.is_empty() || tokens.iter().any(|x| x.is_empty()) {
            return Err(Error::msg("Empty token list!"));
        }

        let (result, last_token, inferred_tokens, terminate_reason) = {
            let mut out_tokens = Vec::with_capacity(64);

            // Locks state_size slots for the infer
            let _permits = state.0.batch_request.request(states.len());

            // Feed prompt first, at least the first token should be ok
            // or there must be some problem in the infer pipeline
            out_tokens.push(
                infer_and_sample(
                    state.clone(),
                    &states,
                    &transformers,
                    tokens,
                    &sampler,
                    &normalizer,
                    update_prompt,
                    None,
                )
                .await
                .map_err(|e| match e {
                    InferenceInterruption::Exhaustion => Error::msg(
                        "Sampler/transformer is exhausted at the start, inference won't continue.",
                    ),
                    InferenceInterruption::Error(e) => e,
                })?,
            );

            let mut last_token = *out_tokens.last().unwrap();

            loop {
                if let Ok(Ok(output)) = state
                    .0
                    .tokenizer
                    .decode(&out_tokens.as_slice())
                    .map(|x| String::from_utf8(x))
                {
                    // out token must be empty when output, or it will be extremely tricky
                    // to hand over the out token.
                    if state
                        .0
                        .terminals
                        .terminate(&terminal, &output, out_tokens.len())?
                        || last_token == 0
                    {
                        break (output, last_token, out_tokens.len(), "by_terminal");
                    }
                }

                // Not ready, infer next one using last token
                out_tokens.push(
                    match infer_and_sample(
                        state.clone(),
                        &states,
                        &transformers,
                        vec![vec![last_token]; states.len()],
                        &sampler,
                        &normalizer,
                        // In autoregressive generation must follow the rule
                        true,
                        Some(&reset_on_exhaustion),
                    )
                    .await
                    {
                        Ok(token) => token,
                        // Exhausted, so stop infer.
                        Err(InferenceInterruption::Exhaustion) => {
                            break (
                                String::from_utf8_lossy(
                                    &state.0.tokenizer.decode(&out_tokens.as_slice()).unwrap(),
                                )
                                .to_string(),
                                last_token,
                                out_tokens.len(),
                                "by_exhaustion",
                            );
                        }
                        // A sampling/transformation error occurred, inference
                        // is terminated
                        Err(InferenceInterruption::Error(error)) => Err(error)?,
                    },
                );
                last_token = *out_tokens.last().unwrap();
            }
        };

        Ok(serde_json::to_value(InferResponse {
            value: result,
            last_token,
            inferred_tokens,
            end_reason: terminate_reason,
        })?)
    } else {
        Err(Error::msg(
            "Field data is needed to specify infer pipeline!",
        ))
    }
}
