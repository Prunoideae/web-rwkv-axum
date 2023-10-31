use std::sync::Arc;

use anyhow::Result;
use dashmap::{DashMap, DashSet};
use tokio::sync::mpsc;
use web_rwkv::context::Context;

use crate::components::model::{AxumModel, AxumModelState};

use super::pool::{InferPool, InferRequest};

struct InnerState {
    id: String,
    state: AxumModelState,
    context: Context,
    model: Arc<AxumModel>,
}

#[derive(Clone)]
pub struct InferState(Arc<InnerState>);

impl InferState {
    pub fn new(id: String, context: Context, model: Arc<AxumModel>, max_state_size: Option<usize>) -> Self {
        let state = AxumModelState::new_sized(&context, &model, 1, max_state_size);
        Self(Arc::new(InnerState {
            id,
            state,
            context,
            model,
        }))
    }

    #[inline(always)]
    pub fn load_to(&self, pool: &AxumModelState, to: usize) -> Result<()> {
        Ok(self.0.state.blit_batch(pool, 0, to)?)
    }

    #[inline(always)]
    pub fn back_from(&self, pool: &AxumModelState, from: usize) -> Result<()> {
        Ok(pool.blit_batch(&self.0.state, from, 0)?)
    }

    #[inline(always)]
    pub fn get_id<'a>(&'a self) -> &'a String {
        &self.0.id
    }

    pub fn clone_new(&self, id: String) -> Result<Self> {
        let new_state = AxumModelState::new(&self.0.context, &self.0.model, 1);
        self.load_to(&new_state, 0)?;
        Ok(Self(Arc::new(InnerState {
            id,
            state: new_state,
            context: self.0.context.clone(),
            model: self.0.model.clone(),
        })))
    }
}

impl PartialEq for InferState {
    // Must ensure that each state is unique.
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

pub struct InnerStates {
    pub context: Context,
    pub model: Arc<AxumModel>,
    pub pool: InferPool,
    pub state_ids: DashSet<String>,
    pub states: DashMap<String, InferState>,
    pub request_queue: mpsc::Sender<Vec<InferRequest>>,
    pub max_state_size: Option<usize>,
}
