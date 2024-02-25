use anyhow::{Error, Result};
use half::f16;
use itertools::Itertools;
use web_rwkv::{
    context::Context,
    model::{
        run::ModelRun, softmax::ModelSoftmax, v4, v5, Build, ModelBase, ModelInfo, ModelInput,
        ModelOutput, ModelState, StateBuilder,
    },
};

#[derive(Debug, Clone)]
pub enum AxumModelState {
    V4(v4::ModelState),
    V5(v5::ModelState),
}

#[derive(Debug, Clone)]
pub enum AxumBackedState {
    V4(v4::BackedState),
    V5(v5::BackedState),
}

impl AxumModelState {
    pub fn new(context: &Context, model: &AxumModel, batch_size: usize) -> Self {
        match model {
            AxumModel::V4(model) => Self::V4(
                StateBuilder::new(context, model.info())
                    .with_num_batch(batch_size)
                    .build()
                    .unwrap(),
            ),
            AxumModel::V5(model) => Self::V5(
                StateBuilder::new(context, model.info())
                    .with_num_batch(batch_size)
                    .build()
                    .unwrap(),
            ),
        }
    }

    pub fn new_sized(
        context: &Context,
        model: &AxumModel,
        batch_size: usize,
        chunk_size: Option<usize>,
    ) -> Self {
        match model {
            AxumModel::V4(model) => Self::V4(
                StateBuilder::new(context, model.info())
                    .with_chunk_size(chunk_size.unwrap_or(model.info().num_layer))
                    .with_num_batch(batch_size)
                    .build()
                    .unwrap(),
            ),
            AxumModel::V5(model) => Self::V5(
                StateBuilder::new(context, model.info())
                    .with_chunk_size(chunk_size.unwrap_or(model.info().num_layer))
                    .with_num_batch(batch_size)
                    .build()
                    .unwrap(),
            ),
        }
    }

    pub fn blit_batch(
        &self,
        dst: &AxumModelState,
        src_index: usize,
        dst_index: usize,
    ) -> Result<()> {
        match (self, dst) {
            (AxumModelState::V4(state), AxumModelState::V4(dst)) => {
                Ok(state.blit_batch(dst, src_index, dst_index)?)
            }
            (AxumModelState::V5(state), AxumModelState::V5(dst)) => {
                Ok(state.blit_batch(dst, src_index, dst_index)?)
            }
            _ => Err(Error::msg("Mismatched state type!")),
        }
    }
}

impl AxumBackedState {
    pub fn new(context: &Context, model: &AxumModel, chunk_size: Option<usize>) -> Self {
        match model {
            AxumModel::V4(model) => Self::V4(
                StateBuilder::new(context, model.info())
                    .with_chunk_size(chunk_size.unwrap_or(model.info().num_layer))
                    .build()
                    .unwrap(),
            ),
            AxumModel::V5(model) => Self::V5(
                StateBuilder::new(context, model.info())
                    .with_chunk_size(chunk_size.unwrap_or(model.info().num_layer))
                    .build()
                    .unwrap(),
            ),
        }
    }

    pub fn load_to(&self, dst: &AxumModelState, dst_index: usize) -> Result<()> {
        match (self, dst) {
            (AxumBackedState::V4(state), AxumModelState::V4(model_state)) => {
                model_state.load_batch(state, dst_index).map_err(Into::into)
            }
            (AxumBackedState::V5(state), AxumModelState::V5(model_state)) => {
                model_state.load_batch(state, dst_index).map_err(Into::into)
            }
            _ => Err(Error::msg("Mismatched state type!")),
        }
    }

    pub async fn back_from(dst: &AxumModelState, dst_index: usize) -> Result<AxumBackedState> {
        match dst {
            AxumModelState::V4(dst) => Ok(AxumBackedState::V4(dst.back_batch(dst_index).await?)),
            AxumModelState::V5(dst) => Ok(AxumBackedState::V5(dst.back_batch(dst_index).await?)),
        }
    }
}

pub enum AxumModel {
    V4(v4::Model<'static, f16>),
    V5(v5::Model<'static, f16>),
}

impl AxumModel {
    pub async fn run(
        &self,
        tokens: &mut Vec<ModelInput>,
        state: &AxumModelState,
    ) -> Result<Vec<ModelOutput>> {
        match &self {
            Self::V4(model) => {
                if let AxumModelState::V4(state) = state {
                    model.run(tokens, state).await.map_err(Into::into)
                } else {
                    Err(Error::msg("Mismatched state type!"))
                }
            }
            Self::V5(model) => {
                if let AxumModelState::V5(state) = state {
                    model.run(tokens, state).await.map_err(Into::into)
                } else {
                    Err(Error::msg("Mismatched state type!"))
                }
            }
        }
    }

    pub fn info(&self) -> &ModelInfo {
        match self {
            AxumModel::V4(model) => model.info(),
            AxumModel::V5(model) => model.info(),
        }
    }

    pub async fn softmax(&self, input: Vec<Vec<f32>>) -> Result<Vec<Vec<f32>>> {
        let input = input
            .into_iter()
            .map(|x| ModelOutput::Last(x))
            .collect_vec();
        Ok(match self {
            AxumModel::V4(model) => model.softmax(input).await,
            AxumModel::V5(model) => model.softmax(input).await,
        }?
        .into_iter()
        .map(|x| match x {
            ModelOutput::Last(x) => x,
            _ => unreachable!(),
        })
        .collect())
    }

    pub async fn infer(
        &self,
        tokens: &mut Vec<ModelInput>,
        state: &AxumModelState,
    ) -> Result<Vec<ModelOutput>> {
        loop {
            let out = self.run(tokens, state).await?;
            if out.iter().any(|out| out.is_some()) {
                break Ok(out);
            }
        }
    }
}
