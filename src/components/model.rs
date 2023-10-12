use anyhow::{Error, Result};
use itertools::Itertools;
use web_rwkv::{
    context::Context,
    model::{v4, v5, Model, ModelInfo, ModelState, StateBuilder},
};

#[derive(Debug, Clone)]
pub enum TypelessModelState {
    V4(v4::ModelState),
    V5(v5::ModelState),
}

impl TypelessModelState {
    pub fn new(context: &Context, model: &TypelessModel, batch_size: usize) -> Self {
        match model {
            TypelessModel::V4(model) => Self::V4(
                StateBuilder::new(context, model.info())
                    .with_max_batch(batch_size)
                    .build(),
            ),
            TypelessModel::V5(model) => Self::V5(
                StateBuilder::new(context, model.info())
                    .with_max_batch(batch_size)
                    .build(),
            ),
        }
    }

    pub fn blit_batch(
        &self,
        dst: &TypelessModelState,
        src_index: usize,
        dst_index: usize,
    ) -> Result<()> {
        match self {
            TypelessModelState::V4(state) => {
                if let TypelessModelState::V4(dst) = dst {
                    Ok(state.blit_batch(dst, src_index, dst_index)?)
                } else {
                    Err(Error::msg("Mismatched state type!"))
                }
            }
            TypelessModelState::V5(state) => {
                if let TypelessModelState::V5(dst) = dst {
                    Ok(state.blit_batch(dst, src_index, dst_index)?)
                } else {
                    Err(Error::msg("Mismatched state type!"))
                }
            }
        }
    }
}

pub enum TypelessModel {
    V4(v4::Model<'static>),
    V5(v5::Model<'static>),
}

impl TypelessModel {
    pub fn run(
        &self,
        tokens: &mut Vec<Vec<u16>>,
        state: &TypelessModelState,
    ) -> Result<Vec<Option<Vec<f32>>>> {
        match &self {
            Self::V4(model) => {
                if let TypelessModelState::V4(state) = state {
                    model.run(tokens, state)
                } else {
                    Err(Error::msg("Mismatched state type!"))
                }
            }
            Self::V5(model) => {
                if let TypelessModelState::V5(state) = state {
                    model.run(tokens, state)
                } else {
                    Err(Error::msg("Mismatched state type!"))
                }
            }
        }
    }

    pub fn info(&self) -> &ModelInfo {
        match self {
            TypelessModel::V4(model) => model.info(),
            TypelessModel::V5(model) => model.info(),
        }
    }

    pub fn softmax(&self, input: Vec<Vec<f32>>) -> Result<Vec<Vec<f32>>> {
        let input = input.into_iter().map(|x| Some(x)).collect_vec();
        Ok(match self {
            TypelessModel::V4(model) => model.softmax(input),
            TypelessModel::V5(model) => model.softmax(input),
        }?
        .into_iter()
        .map(|x| x.unwrap())
        .collect())
    }
}
