use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Error, Result};
use dashmap::DashMap;
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, hashmap_ex};

use self::types::Normalizer;

use super::InferenceInterruption;

pub mod types;

#[derive(Debug, Deserialize)]
struct NormalizerJson {
    type_id: String,
    params: Option<Value>,
}

#[derive()]
pub struct Normalizers {
    registry: HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Normalizer>>>,
    map: DashMap<String, Arc<Mutex<Box<dyn Normalizer>>>>,
}

impl Normalizers {
    pub fn new() -> Self {
        Normalizers {
            registry: hashmap_ex! {
                HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Normalizer>>>,
                    {

                    }
            },
            map: DashMap::with_capacity(128),
        }
    }

    #[inline(always)]
    fn create(
        &self,
        key: &str,
        state: AppState,
        data: Option<Value>,
    ) -> Result<Box<dyn Normalizer>> {
        let constructor = self.registry.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Normalizer not found!"))
        }
    }

    pub fn create_normalizer(&self, id: String, state: AppState, data: Value) -> Result<()> {
        if self.map.contains_key(&id) {
            return Err(Error::msg("Normalizer already existed!"));
        }
        let NormalizerJson { type_id, params } = serde_json::from_value::<NormalizerJson>(data)?;
        self.map.insert(
            id,
            Arc::new(Mutex::new(self.create(&type_id, state, params)?)),
        );
        Ok(())
    }

    #[inline(always)]
    pub fn get_normalizer<'a>(&'a self, id: &str) -> Option<Arc<Mutex<Box<dyn Normalizer>>>> {
        self.map.get(id).map(|x| x.clone())
    }

    #[inline(always)]
    pub fn has_normalizer(&self, id: &str) -> bool {
        self.map.contains_key(id)
    }

    pub fn delete_normalizer(&self, id: &str) -> Result<()> {
        self.map
            .remove(id)
            .ok_or(Error::msg("Normalizer id doesn't exist!"))
            .map(|_| ())
    }

    pub fn reset_normalizer(&self, id: &str) -> Result<()> {
        if let Some(normalizer) = self.map.get(id) {
            normalizer.lock().unwrap().clear();
            Ok(())
        } else {
            Err(Error::msg("Normalizer id doesn't exist!"))
        }
    }

    pub fn update_normalizer(
        &self,
        id: &str,
        content: &Vec<Vec<u16>>,
    ) -> Result<(), InferenceInterruption> {
        if let Some(normalizer) = self.map.get(id) {
            normalizer.lock().unwrap().update(content)
        } else {
            Err(InferenceInterruption::Error(Error::msg(
                "Normalizer id doesn't exist!",
            )))
        }
    }

    pub fn copy_normalizer(&self, src: String, dst: String) -> Result<()> {
        if self.map.contains_key(&dst) {
            return Err(Error::msg("Destination normalizer id already exists!"));
        }
        let src = self
            .map
            .get(&src)
            .ok_or(Error::msg("Normalizer doesn't exist!"))?
            .clone();
        self.map.insert(dst, src);
        Ok(())
    }
}
