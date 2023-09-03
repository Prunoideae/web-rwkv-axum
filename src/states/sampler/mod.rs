use self::types::Sampler;
use crate::{app::SharedState, hashmap_ex, helper::Logits};
use anyhow::{Error, Ok, Result};
use dashmap::DashMap;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub mod types;
pub mod typical;

#[derive(Debug, Deserialize)]
struct SamplerJson {
    type_id: String,
    params: Option<Value>,
}

#[derive(Debug)]
pub struct Samplers {
    registry: HashMap<&'static str, fn(SharedState, Option<Value>) -> Result<Box<dyn Sampler>>>,
    map: DashMap<String, Box<dyn Sampler>>,
}

impl Samplers {
    pub fn new() -> Self {
        Samplers {
            registry: hashmap_ex! {
                HashMap<&'static str, fn(SharedState, Option<Value>) -> Result<Box<dyn Sampler>>>,
                    {
                        "typical" => typical::initialize_typical
                    }
            },
            map: DashMap::with_capacity(128),
        }
    }

    fn create(
        &self,
        key: &str,
        state: SharedState,
        data: Option<Value>,
    ) -> Result<Box<dyn Sampler>> {
        let constructor = self.registry.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Sampler not found!"))
        }
    }

    pub async fn create_sampler(
        &self,
        id: String,
        state: SharedState,
        data: Option<Value>,
    ) -> Result<()> {
        if self.map.contains_key(&id) {
            return Err(Error::msg("Sampler already existed!"));
        }
        if let Some(data) = data {
            let SamplerJson { type_id, params } = serde_json::from_value::<SamplerJson>(data)?;
            self.map.insert(id, self.create(&type_id, state, params)?);
            Ok(())
        } else {
            Err(Error::msg("No data to construct sampler!"))
        }
    }

    pub async fn delete_sampler(&self, id: String) -> Result<()> {
        self.map
            .remove(&id)
            .ok_or(Error::msg("Sampler id doesn't exist!"))
            .map(|_| ())
    }

    pub async fn reset_sampler(&self, id: String) -> Result<()> {
        if let Some(mut sampler) = self.map.get_mut(&id) {
            sampler.clear();
            Ok(())
        } else {
            Err(Error::msg("Sampler id doesn't exist!"))
        }
    }

    pub async fn update_sampler(&self, id: String, content: Vec<u16>) -> Result<()> {
        if let Some(mut sampler) = self.map.get_mut(&id) {
            sampler.update(content);
            Ok(())
        } else {
            Err(Error::msg("Sampler id doesn't exist!"))
        }
    }

    pub async fn copy_sampler(&self, src: String, dst: String) -> Result<()> {
        if self.map.contains_key(&dst) {
            return Err(Error::msg("Destination sampler id already exists!"));
        }
        let src = self
            .map
            .get(&src)
            .ok_or(Error::msg("Sampler doesn't exist!"))?
            .clone();
        self.map.insert(dst, src);
        Ok(())
    }

    pub fn sample_token(&self, id: String, probs: Vec<Logits>) -> Result<u16> {
        if let Some(mut sampler) = self.map.get_mut(&id) {
            Ok(sampler.sample(probs)?)
        } else {
            Err(Error::msg("Sampler id doesn't exist!"))
        }
    }
}
