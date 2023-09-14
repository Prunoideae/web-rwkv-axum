use super::types::Sampler;
use crate::{app::AppState, components::InferenceInterruption};
use anyhow::{Error, Result};
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;

/// Typical sampler for logits
#[derive(Debug, Clone, Deserialize)]
pub struct TypicalSampler {
    top_p: f32,
    temp: f32,
}

impl Sampler for TypicalSampler {
    fn sample(&self, probs: Vec<Vec<f32>>) -> u16 {
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
        token as u16
    }

    fn clear(&mut self) {}

    fn update(&mut self, _tokens: &Vec<Vec<u16>>) -> Result<(), InferenceInterruption> {
        Ok(())
    }

    fn clone(&self) -> Box<dyn Sampler> {
        Box::new(Self {
            top_p: self.top_p,
            temp: self.temp,
        })
    }
}

pub fn initialize_typical(_state: AppState, data: Option<Value>) -> Result<Box<dyn Sampler>> {
    Ok(Box::new(serde_json::from_value::<TypicalSampler>(
        data.ok_or(Error::msg("Field must present to specify top_p and temp!"))?,
    )?))
}
