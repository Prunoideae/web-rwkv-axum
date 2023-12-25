use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::{pipeline::Pipeline, IdParam};

#[derive(Debug, Deserialize)]
pub struct ReplaceTransformer {
    type_id: String,
    params: Option<Value>,
    state_index: usize,
    transformer_index: usize,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTransformer {
    state_index: usize,
    transformer_index: usize,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "modification", rename_all = "snake_case")]
pub enum Modification {
    ReplaceTransformer(ReplaceTransformer),
    ReplaceSampler(IdParam),
    ReplaceTerminal(IdParam),
    DeleteTransformer(DeleteTransformer),
}

impl Modification {
    pub fn modify(self, pipeline: &mut Pipeline, state: &AppState) -> Result<()> {
        match self {
            Modification::ReplaceTransformer(ReplaceTransformer {
                type_id,
                params,
                state_index,
                transformer_index,
            }) => {
                let to_be_modified = pipeline
                    .transformers
                    .get_mut(state_index)
                    .ok_or(Error::msg("State slot does not exist!"))?;

                let transformer =
                    state
                        .0
                        .registry
                        .create_transformer(&type_id, state.clone(), params)?;
                if to_be_modified.len() >= transformer_index {
                    to_be_modified[transformer_index] = transformer;
                } else {
                    to_be_modified.push(transformer)
                }
            }
            Modification::ReplaceSampler(IdParam { type_id, params }) => {
                let sampler = state
                    .0
                    .registry
                    .create_sampler(&type_id, state.clone(), params)?;
                pipeline.sampler = sampler;
            }
            Modification::ReplaceTerminal(IdParam { type_id, params }) => {
                let terminal = state
                    .0
                    .registry
                    .create_terminal(&type_id, state.clone(), params)?;
                pipeline.terminal = terminal;
            }
            Modification::DeleteTransformer(DeleteTransformer {
                state_index,
                transformer_index,
            }) => {
                let to_be_removed = pipeline
                    .transformers
                    .get_mut(state_index)
                    .ok_or(Error::msg("State slot does not exist!"))?;
                if to_be_removed.len() >= transformer_index {
                    to_be_removed.remove(transformer_index);
                } else {
                    to_be_removed.pop();
                }
            }
        }
        Ok(())
    }
}
