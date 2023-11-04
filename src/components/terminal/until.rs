use std::collections::HashSet;

use anyhow::{Error, Result};
use nohash_hasher::BuildNoHashHasher;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::types::Terminal;

#[derive(Debug, Clone, Deserialize)]
pub struct UntilTerminal {
    until: HashSet<u16, BuildNoHashHasher<u16>>,
}

impl Terminal for UntilTerminal {
    fn terminate(&mut self, result: &Vec<u16>, _token_count: usize) -> Result<bool> {
        Ok(result.last().map_or(false, |x| self.until.contains(x)))
    }

    fn clone(&self) -> Box<dyn Terminal> {
        Box::new(UntilTerminal {
            until: self.until.clone(),
        })
    }
}

pub fn intialize_until(state: AppState, data: Option<Value>) -> Result<Box<dyn Terminal>> {
    let until = serde_json::from_value::<String>(
        data.ok_or(Error::msg("Field must present to specify data!"))?,
    )?;

    let until = state
        .0
        .tokenizer
        .token_index_to_bytes()
        .par_iter()
        .enumerate()
        .filter(|(_, bytes)| String::from_utf8_lossy(bytes).contains(&until))
        .map(|(index, _)| index as u16)
        .collect::<HashSet<u16, BuildNoHashHasher<u16>>>();

    Ok(Box::new(UntilTerminal { until }))
}
