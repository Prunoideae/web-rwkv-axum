use anyhow::{Error, Result};
use std::sync::{Arc, Mutex};

use web_rwkv::context::Context;

use crate::components::model::{AxumBackedState, AxumModel, AxumModelState};

struct InnerState {
    id: String,
    state: Mutex<AxumBackedState>,
}

#[derive(Clone)]
pub struct InferState(Arc<InnerState>);

impl InferState {
    pub fn new(
        id: String,
        context: Context,
        model: Arc<AxumModel>,
        chunk_size: Option<usize>,
    ) -> Self {
        Self(Arc::new(InnerState {
            id,
            state: Mutex::new(AxumBackedState::new(&context, &model, chunk_size)),
        }))
    }

    pub fn load_to(&self, pool: &AxumModelState, to: usize) {
        self.0.state.lock().unwrap().load_to(pool, to).unwrap();
    }

    pub fn back_from(&self, pool: &AxumModelState, from: usize) {
        *(self.0.state.lock().unwrap()) = AxumBackedState::back_from(pool, from).unwrap();
    }

    #[inline(always)]
    pub fn get_id<'a>(&'a self) -> &'a String {
        &self.0.id
    }

    pub fn clone_new(&self, id: String) -> Result<Self> {
        Ok(Self(Arc::new(InnerState {
            id,
            state: Mutex::new(
                self.0
                    .state
                    .lock()
                    .map_err(|_| Error::msg("Lock poisoned!"))?
                    .clone(),
            ),
        })))
    }
}

impl PartialEq for InferState {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
