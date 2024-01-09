use anyhow::{Error, Result};
use ndarray::{self, s, Array};
use rand::distributions::{Distribution, WeightedIndex};
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, components::sampler::utils::argsort};

use super::{types::Sampler, utils};

#[derive(Debug, Deserialize)]
pub struct TypicalSampler {
    tau: f32,
    temp: f32,
}

impl TypicalSampler {
    pub fn initialize(_state: AppState, data: Option<Value>) -> Result<Box<dyn Sampler>> {
        let data = serde_json::from_value::<TypicalSampler>(data.ok_or(Error::msg(
            "Invalid typical sampler data. Example format:{
                tau: f32,
                temp: f32,
            }",
        ))?)?;
        if data.temp == 0.0 {
            return Err(Error::msg("data.temp must be larger than 0!"));
        }
        Ok(Box::new(data))
    }
}

impl Sampler for TypicalSampler {
    fn sample(&self, probs: Vec<Vec<f32>>) -> u16 {
        let probs = probs[0].clone();
        let mut probs = Array::from_vec(probs);
        let mut logits = probs.clone();
        logits.par_mapv_inplace(|x| -x.ln());
        let entropy = (&probs * &logits).fold(0.0, |x, y| if y.is_nan() { x } else { x + y });
        logits -= entropy;
        logits.par_mapv_inplace(|x| if x.is_nan() { f32::MAX } else { x.abs() });
        let sorted_ids = argsort(logits.view());
        utils::sort_by_indices(logits.view_mut(), sorted_ids.view());
        utils::sort_by_indices(probs.view_mut(), sorted_ids.view());
        let mut temp = 0.0;
        let cut_off = probs
            .iter()
            .position(|x| {
                temp += x;
                temp >= self.tau
            })
            .unwrap_or(probs.len() - 1);
        let mut probs = probs.slice_move(s![..cut_off + 1]);
        if self.temp != 1.0 {
            probs.par_mapv_inplace(|x| x.powf(1.0 / self.temp));
        }
        let mut rng = rand::thread_rng();
        let token_id = if probs.len() == 1 {
            sorted_ids[0] as u16
        } else {
            sorted_ids[match WeightedIndex::new(probs.as_slice().unwrap()) {
                Ok(index) => index.sample(&mut rng),
                Err(_) => 0,
            }] as u16
        };
        token_id
    }

    fn clone(&self) -> Box<dyn Sampler> {
        Box::new(TypicalSampler {
            tau: self.tau,
            temp: self.temp,
        })
    }
}
