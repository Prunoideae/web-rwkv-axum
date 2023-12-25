use anyhow::Result;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use web_rwkv::context::Context;

use crate::components::model::{AxumBackedState, AxumModel, AxumModelState};

use super::serde;

struct InnerState {
    id: String,
    state: Arc<RwLock<AxumBackedState>>,
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
        }))
    }

    pub async fn new_from(id: String, path: PathBuf) -> Result<Self> {
        let state = serde::load_state(path).await?;
        Ok(Self(Arc::new(InnerState {
            id,
            state: Arc::new(RwLock::new(state)),
        })))
    }

    pub fn load_to(&self, pool: &AxumModelState, to: usize) {
        self.0.state.blocking_read().load_to(pool, to).unwrap();
    }

    pub async fn load_to_async(&self, pool: &AxumModelState, to: usize) {
        self.0.state.read().await.load_to(pool, to).unwrap();
    }

    pub fn back_from(&self, pool: &AxumModelState, from: usize) {
        *(self.0.state.blocking_write()) = AxumBackedState::back_from(pool, from).unwrap();
    }

    pub async fn back_from_async(&self, pool: &AxumModelState, from: usize) {
        *(self.0.state.write().await) = AxumBackedState::back_from(pool, from).unwrap();
    }

    pub async fn dump(&self, path: PathBuf) -> Result<()> {
        let lock = self.0.state.read().await;
        serde::dump_state(&lock, path).await
    }

    #[inline(always)]
    pub fn get_id<'a>(&'a self) -> &'a String {
        &self.0.id
    }

    pub async fn clone_new(&self, id: String) -> Result<Self> {
        Ok(Self(Arc::new(InnerState {
            id,
            state: Arc::new(RwLock::new(self.0.state.read().await.clone())),
        })))
    }

    pub async fn clone_new_async(&self, id: String) -> Result<Self> {
        Ok(Self(Arc::new(InnerState {
            id,
            state: Arc::new(RwLock::new(self.0.state.read().await.clone())),
        })))
    }

    pub fn clone_shallow(&self, id: String) -> Result<Self> {
        Ok(Self(Arc::new(InnerState {
            id,
            state: self.0.state.clone(),
        })))
    }
}

impl PartialEq for NamedState {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}
