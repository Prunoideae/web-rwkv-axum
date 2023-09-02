use self::types::Sampler;
use crate::{app::SharedState, hashmap_ex};
use anyhow::{Error, Ok, Result};
use serde_json::Value;
use std::collections::HashMap;

pub mod types;
pub mod typical;

#[derive(Debug)]
pub struct Samplers {
    map: HashMap<&'static str, fn(SharedState, Option<Value>) -> Result<Box<dyn Sampler>>>,
}

impl Samplers {
    pub fn new() -> Self {
        Samplers {
            map: hashmap_ex! {
                HashMap<&'static str, fn(SharedState, Option<Value>) -> Result<Box<dyn Sampler>>>,
                {
                "typical" => typical::initialize_typical
            }},
        }
    }

    pub fn create_sampler(
        &self,
        key: &str,
        state: SharedState,
        data: Option<Value>,
    ) -> Result<Box<dyn Sampler>> {
        let constructor = self.map.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Sampler not found!"))
        }
    }
}
