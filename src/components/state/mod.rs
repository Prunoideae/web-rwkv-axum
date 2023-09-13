use self::state::{InferState, InnerStates};
use crate::config::ModelConfig;
use anyhow::{Error, Result};
use dashmap::{DashMap, DashSet};
use std::sync::Arc;

mod pool;
mod state;

#[derive(Clone)]
pub struct InferStates(Arc<InnerStates>);

impl InferStates {
    pub async fn new(config: &ModelConfig) -> Result<Self> {
        let context = config.model.create_context().await?;
        let model = Arc::new(config.model.load_model(&context).await?);
        let batch_size = config.model.get_batch_size();

        Ok(Self(Arc::new(InnerStates {
            context,
            model,
            pool: todo!(),
            state_ids: DashSet::with_capacity(128),
            states: DashMap::with_capacity(128),
        })))
    }

    pub async fn infer(
        &self,
        states: &Vec<String>,
        tokens: Vec<Vec<u16>>,
    ) -> Result<Vec<Vec<f32>>> {
        todo!()
    }

    pub fn create_state(&self, state_id: &str) -> Result<()> {
        if self.0.state_ids.contains(state_id) {
            return Err(Error::msg("State already exists!"));
        }
        self.0.state_ids.insert(state_id.to_string());
        Ok(())
    }

    pub fn copy_state(&self, src: &str, dst: &str) -> Result<()> {
        if self.0.state_ids.contains(dst) {
            return Err(Error::msg("Destination state already exists!"));
        }
        if !self.0.state_ids.contains(src) {
            return Err(Error::msg("Source state id doesn't exist!"));
        }

        if self.0.states.contains_key(src) {
            tokio::task::block_in_place(|| {
                self.0.pool.sync(src)?;
                let dst_state = self.0.states.get(src).unwrap().clone_new(dst.to_string())?;
                self.0.states.insert(dst.to_string(), dst_state);
                Ok::<(), Error>(())
            })?;
        }
        Ok(())
    }

    pub fn delete_state(&self, state_id: &str) -> Result<()> {
        match self.0.state_ids.remove(state_id) {
            Some(_) => {
                self.0.states.remove(state_id);
                Ok(())
            }
            None => Err(Error::msg("State ID does not exist!")),
        }
    }

    pub fn get_state(&self, state_id: &str) -> InferState {
        self.0
            .states
            .entry(state_id.to_string())
            .or_insert_with(|| {
                InferState::new(state_id.to_string(), self.0.context.clone(), &self.0.model)
            })
            .clone()
    }
}
