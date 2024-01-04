use crate::app::AppState;

use crate::components::{
    infer::updates::{ResetSetting, UpdateSetting},
    normalizer::types::Normalizer,
    sampler::types::Sampler,
    state::InferTicket,
    terminal::types::Terminal,
    transformer::types::Transformer,
    InferenceInterruption,
};
use anyhow::{Error, Result};
use itertools::Itertools;
use rayon::prelude::*;

pub struct Pipeline {
    pub(super) transformers: Vec<Vec<Box<dyn Transformer>>>,
    pub(super) sampler: Box<dyn Sampler>,
    pub(super) terminal: Box<dyn Terminal>,
    pub(super) normalizer: Option<Box<dyn Normalizer>>,
}

impl Clone for Pipeline {
    fn clone(&self) -> Self {
        Self {
            transformers: self
                .transformers
                .par_iter()
                .map(|x| x.iter().map(|x| x.as_ref().clone()).collect_vec())
                .collect::<Vec<_>>(),
            sampler: self.sampler.clone(),
            terminal: self.terminal.clone(),
            normalizer: self.normalizer.as_ref().map(|x| x.as_ref().clone()),
        }
    }
}

impl Pipeline {
    pub fn new(
        transformers: Vec<Vec<Box<dyn Transformer>>>,
        sampler: Box<dyn Sampler>,
        terminal: Box<dyn Terminal>,
        normalizer: Option<Box<dyn Normalizer>>,
    ) -> Self {
        Self {
            transformers,
            sampler,
            terminal,
            normalizer,
        }
    }

    pub async fn infer(
        &mut self,
        mut ticket: InferTicket,
        reset_setting: ResetSetting,
        update_setting: UpdateSetting,
        tokens: Vec<Vec<u16>>,
        max_tokens: usize,
        state: &AppState,
    ) -> Result<(u16, Vec<u16>, &'static str)> {
        self.update_prompt(&tokens, update_setting)?;

        let logits = ticket.infer(tokens).await;
        let state_count = ticket.state_size();
        let mut last_token = self.sample(logits, &state).await;
        let mut inferred_tokens = vec![last_token];

        let end_reason = loop {
            if self.terminate(&inferred_tokens, inferred_tokens.len())? {
                break "by_terminal";
            }
            if last_token == 0 {
                break "by_eos";
            }
            if inferred_tokens.len() >= max_tokens {
                break "by_max_tokens";
            }

            let token_vec = vec![vec![last_token]; state_count];
            match self.update_auto(&token_vec) {
                Ok(_) => (),
                Err(InferenceInterruption::Exhaustion) => {
                    self.reset(reset_setting);
                    break "by_exhaustion";
                }
                Err(InferenceInterruption::Error(e)) => Err(e)?,
            }
            let logits = ticket.infer(token_vec).await;
            last_token = self.sample(logits, &state).await;
            inferred_tokens.push(last_token)
        };

        Ok((last_token, inferred_tokens, end_reason))
    }

    pub fn reset_all(&mut self) {
        self.sampler.clear();
        self.terminal.clear();
        if let Some(n) = self.normalizer.as_mut() {
            n.clear()
        }
        self.transformers
            .iter_mut()
            .flatten()
            .for_each(|t| t.clear());
    }

    pub fn get_transformer_shape(&self) -> Vec<Vec<()>> {
        self.transformers
            .iter()
            .map(|x| x.iter().map(|_| ()).collect())
            .collect()
    }
}

impl Pipeline {
    fn update_auto(&mut self, tokens: &Vec<Vec<u16>>) -> Result<(), InferenceInterruption> {
        self.sampler.update(tokens)?;
        if let Some(normalizer) = self.normalizer.as_mut() {
            normalizer.update(tokens)?;
        }
        self.transformers
            .par_iter_mut()
            .zip(tokens.par_iter())
            .map(|(transformers, tokens)| {
                for transformer in transformers {
                    transformer.update(tokens)?;
                }
                Ok(())
            })
            .collect::<Result<Vec<_>, InferenceInterruption>>()?;

        Ok(())
    }

    fn update_prompt(
        &mut self,
        tokens: &Vec<Vec<u16>>,
        update_setting: UpdateSetting,
    ) -> Result<()> {
        let UpdateSetting {
            transformers,
            sampler,
            normalizer,
        } = update_setting;

        if sampler {
            self.sampler.update_prompt(tokens)?;
        }

        if normalizer {
            if let Some(normalizer) = self.normalizer.as_mut() {
                normalizer.update_prompt(tokens)?;
            }
        }

        self.transformers
            .par_iter_mut()
            .zip(tokens.par_iter())
            .zip(transformers.into_par_iter())
            .map(|((transformers, tokens), updates)| {
                for (transformer, update) in transformers.iter_mut().zip(updates.into_iter()) {
                    if update {
                        transformer.update_prompt(tokens)?;
                    }
                }
                Ok(())
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(())
    }

    fn reset(&mut self, reset_setting: ResetSetting) {
        let ResetSetting {
            transformers,
            sampler,
            normalizer,
        } = reset_setting;

        self.transformers
            .iter_mut()
            .flatten()
            .zip(transformers)
            .filter(|(_, x)| *x)
            .for_each(|(x, _)| x.clear());

        if sampler {
            self.sampler.clear();
        }

        if normalizer {
            if let Some(normalizer) = self.normalizer.as_mut() {
                normalizer.clear()
            }
        }
    }

    #[inline(always)]
    pub fn terminate(&mut self, result: &Vec<u16>, token_count: usize) -> Result<bool> {
        self.terminal.terminate(result, token_count)
    }

    pub async fn sample(&self, logits: Vec<Vec<f32>>, app_state: &AppState) -> u16 {
        let logits = self
            .transformers
            .par_iter()
            .zip(logits.into_par_iter())
            .map(|(transformers, mut logits)| {
                for transformer in transformers {
                    logits = transformer.transform(logits);
                }
                logits
            })
            .collect::<Vec<_>>();
        let logits = if let Some(normalizer) = &self.normalizer {
            normalizer.normalize(logits)
        } else {
            app_state.softmax(logits).await
        };
        self.sampler.sample(logits)
    }
}
