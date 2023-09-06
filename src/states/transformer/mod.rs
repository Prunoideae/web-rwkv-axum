use self::types::Transformer;
use crate::{app::SharedState, hashmap_ex};
use anyhow::{Error, Ok, Result};
use dashmap::{mapref::one::RefMut, DashMap};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub mod types;

#[derive(Debug, Deserialize)]
struct TransformerJson {
    type_id: String,
    params: Option<Value>,
}

pub struct Transformers {
    registry: HashMap<&'static str, fn(SharedState, Option<Value>) -> Result<Box<dyn Transformer>>>,
    map: DashMap<String, Box<dyn Transformer>>,
}

impl Transformers {
    pub fn new() -> Self {
        Self {
            registry: hashmap_ex! {
                HashMap<&'static str, fn(SharedState, Option<Value>) -> Result<Box<dyn Transformer>>>,
                    {

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
    ) -> Result<Box<dyn Transformer>> {
        let constructor = self.registry.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Transformer not found!"))
        }
    }

    pub fn create_transformer(
        &self,
        id: String,
        state: SharedState,
        data: Option<Value>,
    ) -> Result<()> {
        if self.map.contains_key(&id) {
            return Err(Error::msg("Transformer already existed!"));
        }
        if let Some(data) = data {
            let TransformerJson { type_id, params } =
                serde_json::from_value::<TransformerJson>(data)?;
            self.map.insert(id, self.create(&type_id, state, params)?);
            Ok(())
        } else {
            Err(Error::msg("No data to construct transformer!"))
        }
    }

    #[inline(always)]
    pub fn get_transformer<'a>(
        &'a self,
        id: &String,
    ) -> Option<RefMut<'_, String, Box<dyn Transformer>>> {
        self.map.get_mut(id)
    }

    #[inline(always)]
    pub fn has_transformer(&self, id: &String) -> bool {
        self.map.contains_key(id)
    }

    pub fn delete_transformer(&self, id: String) -> Result<()> {
        self.map
            .remove(&id)
            .ok_or(Error::msg("Transformer id doesn't exist!"))
            .map(|_| ())
    }

    pub async fn reset_transformer(&self, id: String) -> Result<()> {
        if let Some(mut transformer) = self.map.get_mut(&id) {
            transformer.clear();
            Ok(())
        } else {
            Err(Error::msg("Transformer id doesn't exist!"))
        }
    }

    pub fn update_transformer(&self, id: &String, content: &Vec<u16>) -> Result<()> {
        if let Some(mut transformer) = self.map.get_mut(id) {
            transformer.update(content);
            Ok(())
        } else {
            Err(Error::msg("Transformer id doesn't exist!"))
        }
    }

    pub fn copy_transformer(&self, src: String, dst: String) -> Result<()> {
        if self.map.contains_key(&dst) {
            return Err(Error::msg("Destination transformer id already exists!"));
        }
        let src = self
            .map
            .get(&src)
            .ok_or(Error::msg("Transformer doesn't exist!"))?
            .clone();
        self.map.insert(dst, src);
        Ok(())
    }

    pub fn transform_logits(&self, id: &String, logits: &mut Vec<f32>) -> Result<()> {
        if let Some(transformer) = self.map.get_mut(id) {
            Ok(transformer.transform(logits))
        } else {
            Err(Error::msg("Transformer id doesn't exist!"))
        }
    }
}
