use anyhow::{Error, Result};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::types::Transformer;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LogitsCompressor {
    factor: f32,
}

impl LogitsCompressor {
    pub fn initialize(_state: AppState, data: Option<Value>) -> Result<Box<dyn Transformer>> {
        let data = serde_json::from_value::<LogitsCompressor>(
            data.ok_or(Error::msg("Field must present to specify temp!"))?,
        )?;
        Ok(Box::new(data))
    }
}

impl Transformer for LogitsCompressor {
    fn transform(&self, logits: Vec<f32>) -> Vec<f32> {
        logits.par_iter().map(|x| x / self.factor).collect()
    }

    fn clone(&self) -> Box<dyn Transformer> {
        Box::new(*self)
    }
}
