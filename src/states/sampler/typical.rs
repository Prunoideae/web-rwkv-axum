use super::types::Sampler;
use crate::app::AppState;
use anyhow::{Error, Ok, Result};
use itertools::Itertools;
use serde_json::Value;

/// Typical sampler for logits
#[derive(Debug, Clone)]
pub struct TypicalSampler {
    top_p: f32,
    temp: f32,
}

impl Sampler for TypicalSampler {
    fn sample(&self, probs: Vec<Vec<f32>>) -> Result<u16> {
        let probs = &probs[0];
        let sorted = probs
            .into_iter()
            .enumerate()
            .sorted_unstable_by(|(_, x), (_, y)| x.total_cmp(&y).reverse())
            .scan((0, 0.0), |(_, cum), (id, x)| {
                if *cum > self.top_p {
                    None
                } else {
                    *cum += x;
                    Some((id, *cum))
                }
            })
            .collect_vec();
        let sum: f32 = sorted.iter().map(|(_, x)| x).sum();
        let sorted = sorted.into_iter().map(|(id, x)| (id, x / sum));

        let rand = fastrand::f32();
        let token = sorted
            .into_iter()
            .find_or_first(|&(_, cum)| rand <= cum)
            .map(|(id, _)| id)
            .unwrap_or_default();
        Ok(token as u16)
    }

    fn clear(&mut self) {}

    fn update(&mut self, _tokens: &Vec<Vec<u16>>) {}

    fn clone(&self) -> Box<dyn Sampler> {
        Box::new(Self {
            top_p: self.top_p,
            temp: self.temp,
        })
    }
}

pub fn initialize_typical(_state: AppState, data: Option<Value>) -> Result<Box<dyn Sampler>> {
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
