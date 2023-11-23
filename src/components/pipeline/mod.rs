use anyhow::{Error, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

use dashmap::DashMap;
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use self::pipeline::Pipeline;

pub mod pipeline;

#[derive(Debug, Deserialize)]
struct IdParam {
    type_id: String,
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct PipelinePayload {
    id: String,
    transformers: Vec<Vec<IdParam>>,
    sampler: IdParam,
    terminal: IdParam,
    normalizer: Option<IdParam>,
}

pub struct Pipelines {
    map: DashMap<String, Arc<Mutex<Pipeline>>>,
}

impl Pipelines {
    pub fn new() -> Self {
        Self {
            map: DashMap::with_capacity(128),
        }
    }

    pub fn create_pipeline(&self, state: &AppState, payload: Value) -> Result<()> {
        let PipelinePayload {
            id,
            transformers,
            sampler,
            terminal,
            normalizer,
        } = serde_json::from_value(payload)?;

        if self.map.contains_key(&id) {
            return Err(Error::msg("Pipeline id exists!"));
        }

        let sampler =
            state
                .0
                .registry
                .create_sampler(&sampler.type_id, state.clone(), sampler.params)?;

        let terminal =
            state
                .0
                .registry
                .create_terminal(&terminal.type_id, state.clone(), terminal.params)?;

        let normalizer = if let Some(n) = normalizer {
            Some(
                state
                    .0
                    .registry
                    .create_normalizer(&n.type_id, state.clone(), n.params)?,
            )
        } else {
            None
        };

        let transformers = transformers
            .into_iter()
            .map(|x| {
                x.into_iter()
                    .map(|IdParam { type_id, params }| {
                        state
                            .0
                            .registry
                            .create_transformer(&type_id, state.clone(), params)
                    })
                    .collect::<Result<Vec<Box<_>>>>()
            })
            .collect::<Result<Vec<_>>>()?;

        self.map.insert(
            id,
            Arc::new(Mutex::new(Pipeline::new(
                transformers,
                sampler,
                terminal,
                normalizer,
            ))),
        );

        Ok(())
    }

    pub fn remove_pipeline(&self, id: &str) -> Result<()> {
        self.map
            .remove(id)
            .ok_or(Error::msg("Pipeline ID does not exist."))?;
        Ok(())
    }

    pub async fn copy_pipeline(&self, src: &str, dst: String) -> Result<()> {
        if self.map.contains_key(&dst) {
            return Err(Error::msg("Destination pipeline id exists."));
        }

        let pipeline = Arc::new(Mutex::new(
            self.map
                .get(src)
                .ok_or(Error::msg("Source pipeline id does not exist."))?
                .lock()
                .await
                .clone(),
        ));
        self.map.insert(dst, pipeline);

        Ok(())
    }

    pub fn get_pipeline(&self, id: &str) -> Result<Arc<Mutex<Pipeline>>> {
        Ok(self
            .map
            .get(id)
            .ok_or(Error::msg("Source pipeline id does not exist."))?
            .clone())
    }
}
