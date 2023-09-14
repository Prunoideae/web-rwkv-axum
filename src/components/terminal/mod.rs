use anyhow::{Error, Result};
use dashmap::{mapref::one::RefMut, DashMap};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::{app::AppState, hashmap_ex};

use self::types::Terminal;

mod lengthed;
mod types;

#[derive(Debug, Deserialize)]
struct TerminalJson {
    type_id: String,
    params: Option<Value>,
}

pub struct Terminals {
    registry: HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Terminal>>>,
    map: DashMap<String, Box<dyn Terminal>>,
}

impl Terminals {
    pub fn new() -> Self {
        Terminals {
            registry: hashmap_ex! {
                HashMap<&str, fn(AppState, Option<Value>) -> Result<Box<dyn Terminal>>>,
                    {
                        "lengthed" => lengthed::initialize_lenghted
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
        self.map.insert(id, self.create(&type_id, state, params)?);
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
    pub fn get_terminal<'a>(&'a self, id: &str) -> Option<RefMut<'a, String, Box<dyn Terminal>>> {
        self.map.get_mut(id)
    }

    #[inline(always)]
    pub fn has_terminal(&self, id: &str) -> bool {
        self.map.contains_key(id)
    }

    pub fn reset_terminal(&self, id: &str) -> Result<()> {
        if let Some(mut sampler) = self.map.get_mut(id) {
            sampler.clear();
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

    pub fn terminate(&self, id: &str, result: &str, token_count: usize) -> Result<bool> {
        if let Some(mut terminal) = self.get_terminal(id) {
            terminal.terminate(result, token_count)
        } else {
            Err(Error::msg("Terminal id doesn't exist!"))
        }
    }
}
