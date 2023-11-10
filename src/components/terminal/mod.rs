use anyhow::{Error, Result};
use dashmap::DashMap;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{app::AppState, hashmap_ex};

use self::types::Terminal;

mod lengthed;
pub mod types;
mod until;

#[derive(Debug, Deserialize)]
struct TerminalJson {
    type_id: String,
    params: Option<Value>,
}

pub struct Terminals {
    registry: HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Terminal>>>,
    map: DashMap<String, Arc<Mutex<Box<dyn Terminal>>>>,
}

impl Terminals {
    pub fn new() -> Self {
        Terminals {
            registry: hashmap_ex! {
                HashMap<&str, fn(AppState, Option<Value>) -> Result<Box<dyn Terminal>>>,
                    {
                        "lengthed" => lengthed::initialize_lenghted,
                        "until" => until::intialize_until,
                    }
            },
            map: DashMap::with_capacity(128),
        }
    }

    fn create(&self, key: &str, state: AppState, data: Option<Value>) -> Result<Box<dyn Terminal>> {
        let constructor = self.registry.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Terminal not found!"))
        }
    }

    pub fn create_terminal(&self, id: String, state: AppState, data: Value) -> Result<()> {
        if self.map.contains_key(&id) {
            return Err(Error::msg("Terminal already existed!"));
        }
        let TerminalJson { type_id, params } = serde_json::from_value(data)?;
        self.map.insert(
            id,
            Arc::new(Mutex::new(self.create(&type_id, state, params)?)),
        );
        Ok(())
    }

    pub fn delete_terminal(&self, id: &str) -> Result<()> {
        if let None = self.map.remove(id) {
            Err(Error::msg("Terminal id doesn't exist!"))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub fn get_terminal<'a>(&'a self, id: &str) -> Option<Arc<Mutex<Box<dyn Terminal>>>> {
        self.map.get(id).map(|x| x.clone())
    }

    #[inline(always)]
    pub fn has_terminal(&self, id: &str) -> bool {
        self.map.contains_key(id)
    }

    pub fn reset_terminal(&self, id: &str) -> Result<()> {
        if let Some(sampler) = self.map.get(id) {
            sampler.lock().unwrap().clear();
            Ok(())
        } else {
            Err(Error::msg("Terminal id doesn't exist!"))
        }
    }

    pub fn copy_terminal(&self, src: String, dst: String) -> Result<()> {
        if self.map.contains_key(&dst) {
            return Err(Error::msg("Destination terminal id already exists!"));
        }
        let src = self
            .map
            .get(&src)
            .ok_or(Error::msg("Terminal doesn't exist!"))?
            .clone();
        self.map.insert(dst, src);
        Ok(())
    }
}
