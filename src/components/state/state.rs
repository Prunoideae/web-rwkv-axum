use anyhow::{Error, Result};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use web_rwkv::context::Context;

use crate::components::model::{AxumBackedState, AxumModel, AxumModelState};

use super::serde;

struct InnerState {
    id: String,
    state: Arc<RwLock<AxumBackedState>>,
    valid: Mutex<bool>,
}

#[derive(Clone)]
pub struct NamedState(Arc<InnerState>);

impl NamedState {
    pub fn new(
        id: String,
        context: Context,
        model: Arc<AxumModel>,
        chunk_size: Option<usize>,
    ) -> Self {
        Self(Arc::new(InnerState {
            id,
            state: Arc::new(RwLock::new(AxumBackedState::new(
                &context, &model, chunk_size,
            ))),
            valid: Mutex::new(true),
        }))
    }

    pub async fn new_from(id: String, path: PathBuf) -> Result<Self> {
        let state = serde::load_state(path).await?;
        Ok(Self(Arc::new(InnerState {
            id,
            state: Arc::new(RwLock::new(state)),
            valid: Mutex::new(true),
        })))
    }

    pub fn load_to(&self, pool: &AxumModelState, to: usize) {
        self.0.state.read().unwrap().load_to(pool, to).unwrap();
    }

    pub fn back_from(&self, pool: &AxumModelState, from: usize) {
        *(self.0.state.write().unwrap()) = AxumBackedState::back_from(pool, from).unwrap();
    }

    pub async fn dump(&self, path: PathBuf) -> Result<()> {
        serde::dump_state(&self.0.state.read().unwrap(), path).await
    }

    #[inline(always)]
    pub fn get_id<'a>(&'a self) -> &'a String {
        &self.0.id
    }

    pub fn is_valid(&self) -> bool {
        *self.0.valid.lock().unwrap()
    }

    pub fn invalidate(&self) {
        *self.0.valid.lock().unwrap() = false
    }

    pub fn clone_new(&self, id: String) -> Result<Self> {
        Ok(Self(Arc::new(InnerState {
            id,
            state: Arc::new(RwLock::new(
                self.0
                    .state
                    .read()
                    .map_err(|_| Error::msg("Lock poisoned!"))?
                    .clone(),
            )),
            valid: Mutex::new(true),
        })))
    }

    pub fn clone_shallow(&self, id: String) -> Result<Self> {
        Ok(Self(Arc::new(InnerState {
            id,
            state: self.0.state.clone(),
            valid: Mutex::new(true),
        })))
    }
}

impl PartialEq for NamedState {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
