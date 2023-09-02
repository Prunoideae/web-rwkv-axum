use anyhow::{Error, Ok, Result};
use serde_json::Value;

use crate::app::SharedState;

use super::types::Sampler;

/// Typical sampler for logits
#[derive(Debug)]
pub struct TypicalSampler {
    top_p: f32,
    temp: f32,
}

impl Sampler for TypicalSampler {
    fn sample(&mut self, probs: Vec<Vec<f32>>) -> Result<u16> {
        todo!()
    }
}

pub fn initialize_typical(state: SharedState, data: Option<Value>) -> Result<Box<dyn Sampler>> {
    let mut top_p: f32 = 0.6;
    let mut temp: f32 = 1.0;

    if let Some(Value::Object(values)) = data {
        if let Some(new_top_p) = values.get("top_p") {
            top_p = new_top_p
                .as_f64()
                .ok_or(Error::msg("top_p must be a float!"))? as f32;
        }

        if let Some(new_temp) = values.get("temp") {
            temp = new_temp
                .as_f64()
                .ok_or(Error::msg("temp must be a float!"))? as f32;
        }
    }

    Ok(Box::new(TypicalSampler { top_p, temp }))
}
