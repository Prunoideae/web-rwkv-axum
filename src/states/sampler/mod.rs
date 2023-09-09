use self::types::Sampler;
use crate::{app::AppState, hashmap_ex};
use anyhow::{Error, Ok, Result};
use dashmap::{mapref::one::RefMut, DashMap};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use super::InferenceInterruption;

pub mod types;
pub mod typical;

#[derive(Debug, Deserialize)]
struct SamplerJson {
    type_id: String,
    params: Option<Value>,
}

#[derive(Debug)]
pub struct Samplers {
    registry: HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Sampler>>>,
    map: DashMap<String, Box<dyn Sampler>>,
}

impl Samplers {
    pub fn new() -> Self {
        Samplers {
            registry: hashmap_ex! {
                HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Sampler>>>,
                    {
                        "typical" => typical::initialize_typical
                    }
            },
            map: DashMap::with_capacity(128),
        }
    }

    fn create(&self, key: &str, state: AppState, data: Option<Value>) -> Result<Box<dyn Sampler>> {
        let constructor = self.registry.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Sampler not found!"))
        }
    }

    pub fn create_sampler(&self, id: String, state: AppState, data: Value) -> Result<()> {
        if self.map.contains_key(&id) {
            return Err(Error::msg("Sampler already existed!"));
        }
        let SamplerJson { type_id, params } = serde_json::from_value::<SamplerJson>(data)?;
        self.map.insert(id, self.create(&type_id, state, params)?);
        Ok(())
    }

    #[inline(always)]
    pub fn get_sampler<'a>(&'a self, id: &String) -> Option<RefMut<'_, String, Box<dyn Sampler>>> {
        self.map.get_mut(id)
    }

    #[inline(always)]
    pub fn has_sampler(&self, id: &String) -> bool {
        self.map.contains_key(id)
    }

    pub fn delete_sampler(&self, id: String) -> Result<()> {
        self.map
            .remove(&id)
            .ok_or(Error::msg("Sampler id doesn't exist!"))
            .map(|_| ())
    }

    pub fn reset_sampler(&self, id: String) -> Result<()> {
        if let Some(mut sampler) = self.map.get_mut(&id) {
            sampler.clear();
            Ok(())
        } else {
            Err(Error::msg("Sampler id doesn't exist!"))
        }
    }

    pub fn update_sampler(&self, id: &String, content: &Vec<Vec<u16>>) -> Result<(), InferenceInterruption> {
        if let Some(mut sampler) = self.map.get_mut(id) {
            sampler.update(content)
        } else {
            Err(InferenceInterruption::Error(Error::msg("Sampler id doesn't exist!")))
        }
    }

    pub fn copy_sampler(&self, src: String, dst: String) -> Result<()> {
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

    pub fn sample_token(&self, id: &String, probs: Vec<Vec<f32>>) -> Result<u16> {
        if let Some(sampler) = self.map.get(id) {
            Ok(sampler.sample(probs))
        } else {
            Err(Error::msg("Sampler id doesn't exist!"))
        }
    }
}
