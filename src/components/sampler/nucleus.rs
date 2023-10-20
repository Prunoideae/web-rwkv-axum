use super::types::Sampler;
use crate::{
    app::AppState,
    components::sampler::utils::{argsort, sort_by_indices},
};
use anyhow::{Error, Result};
use ndarray::{s, Array};
use rand::{distributions::WeightedIndex, prelude::Distribution};
use serde::Deserialize;
use serde_json::Value;

/// Test sampler for logits
#[derive(Debug, Clone, Deserialize)]
pub struct NucleusSampler {
    top_p: f32,
    temp: f32,
}

impl Sampler for NucleusSampler {
    fn sample(&self, mut probs: Vec<Vec<f32>>) -> u16 {
        let mut probs = Array::from_vec(probs.remove(0));
        let reversed_probs = -probs.clone();
        let sorted_ids = argsort(reversed_probs.view());
        sort_by_indices(probs.view_mut(), sorted_ids.view());
        let mut temp = 0.0;
        let cut_off = probs
            .iter()
            .position(|x| {
                temp += x;
                temp >= self.top_p
            })
            .unwrap_or(probs.len() - 1);
        let mut probs = probs.slice_move(s![..cut_off + 1]);
        if self.temp != 1.0 {
            probs.par_mapv_inplace(|x| x.powf(1.0 / self.temp));
        }
        let mut rng = rand::thread_rng();
        let token_id = sorted_ids[WeightedIndex::new(probs.as_slice().unwrap())
            .unwrap()
            .sample(&mut rng)] as u16;
        token_id
    }

    fn clone(&self) -> Box<dyn Sampler> {
        Box::new(Self {
            top_p: self.top_p,
            temp: self.temp,
        })
    }
}

pub fn initialize(_state: AppState, data: Option<Value>) -> Result<Box<dyn Sampler>> {
    Ok(Box::new(serde_json::from_value::<NucleusSampler>(
        data.ok_or(Error::msg(
            "
        Invalid NucleusSampler data. Example format:{
            top_p: f32,
            temp: f32
        }
        ",
        ))?,
    )?))
}
