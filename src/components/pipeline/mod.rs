use anyhow::{Error, Result};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use self::pipeline::Pipeline;

pub mod mutate;
pub mod pipeline;

#[derive(Debug, Deserialize)]
pub struct IdParam {
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
    map: RwLock<HashMap<String, Arc<Mutex<Pipeline>>>>,
}

impl Pipelines {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::with_capacity(128)),
        }
    }

    pub async fn create_pipeline(&self, state: &AppState, payload: Value) -> Result<()> {
        let PipelinePayload {
            id,
            transformers,
            sampler,
            terminal,
            normalizer,
        } = serde_json::from_value(payload)?;

        if self.has_pipeline(&id).await {
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

        self.set_pipeline(
            &id,
            Pipeline::new(transformers, sampler, terminal, normalizer),
        )
        .await?;

        Ok(())
    }

    pub async fn remove_pipeline(&self, id: &str) -> Result<()> {
        self.map
            .write()
            .await
            .remove(id)
            .ok_or(Error::msg("Pipeline ID does not exist."))?;
        Ok(())
    }

    pub async fn pop_pipeline(&self, id: &str) -> Result<Pipeline> {
        Ok(Arc::try_unwrap(
            self.map
                .write()
                .await
                .remove(id)
                .ok_or(Error::msg("Pipeline ID does not exist."))?,
        )
        .map_err(|_| Error::msg("Pipeline is still held by inferences."))?
        .into_inner())
    }

    pub async fn copy_pipeline(&self, src: &str) -> Result<Pipeline> {
        Ok(self.get_pipeline(src).await?.lock().await.clone())
    }

    pub async fn get_pipeline(&self, id: &str) -> Result<Arc<Mutex<Pipeline>>> {
        Ok(self
            .map
            .read()
            .await
            .get(id)
            .ok_or(Error::msg("Source pipeline id does not exist."))?
            .clone())
    }

    pub async fn set_pipeline(&self, id: &str, pipeline: Pipeline) -> Result<()> {
        if self.has_pipeline(id).await {
            return Err(Error::msg("Pipeline ID exists."));
        }
        self.map
            .write()
            .await
            .insert(id.to_string(), Arc::new(Mutex::new(pipeline)));
        Ok(())
    }

    #[inline(always)]
    pub async fn has_pipeline(&self, id: &str) -> bool {
        self.map.read().await.contains_key(id)
    }
}
