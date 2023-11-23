use anyhow::{Error, Result};
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ResetSetting {
    pub transformers: Vec<bool>,
    pub sampler: bool,
    pub normalizer: bool,
}

#[derive(Debug, Deserialize)]
struct ResetData {
    transformers: Option<Vec<Vec<bool>>>,
    sampler: Option<bool>,
    normalizer: Option<bool>,
}

impl ResetSetting {
    fn all_bool(transformers: &Vec<Vec<()>>, value: bool) -> Self {
        Self {
            transformers: transformers.iter().flatten().map(|_| value).collect_vec(),
            sampler: value,
            normalizer: value,
        }
    }

    pub fn from_value(transformer_ids: &Vec<Vec<()>>, value: Option<Value>) -> Result<Self> {
        if let Some(value) = value {
            match value {
                Value::Bool(flag) => Ok(Self::all_bool(transformer_ids, flag)),
                Value::Object(_) => {
                    let ResetData {
                        transformers,
                        sampler,
                        normalizer,
                    } = serde_json::from_value::<ResetData>(value)?;

                    let sampler = sampler.unwrap_or(true);
                    let normalizer = normalizer.unwrap_or(true);
                    let transformers = transformers.unwrap_or_else(|| {
                        transformer_ids
                            .iter()
                            .map(|x| x.iter().map(|_| true).collect())
                            .collect()
                    });

                    if transformers
                        .iter()
                        .zip(transformer_ids.iter())
                        .any(|(a, b)| a.len() != b.len())
                    {
                        return Err(Error::msg("The shape of transformers and ids must match!"));
                    }
                    Ok(Self {
                        transformers: transformers.into_iter().flatten().collect(),
                        sampler,
                        normalizer,
                    })
                }
                _ => Err(Error::msg("Must be a bool or an object!")),
            }
        } else {
            Ok(Self::all_bool(transformer_ids, true))
        }
    }
}

pub struct UpdateSetting {
    pub transformers: Vec<Vec<bool>>,
    pub sampler: bool,
    pub normalizer: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateData {
    transformers: Option<Vec<Vec<bool>>>,
    sampler: Option<bool>,
    normalizer: Option<bool>,
}

impl UpdateSetting {
    fn all_bool(transformers: &Vec<Vec<()>>, value: bool) -> Self {
        Self {
            transformers: transformers
                .iter()
                .map(|t| t.iter().map(|_| value).collect())
                .collect_vec(),
            sampler: value,
            normalizer: value,
        }
    }

    pub fn from_value(transformer_ids: &Vec<Vec<()>>, value: Option<Value>) -> Result<Self> {
        if let Some(value) = value {
            match value {
                Value::Bool(flag) => Ok(Self::all_bool(transformer_ids, flag)),
                Value::Object(_) => {
                    let UpdateData {
                        transformers,
                        sampler,
                        normalizer,
                    } = serde_json::from_value::<UpdateData>(value)?;

                    let sampler = sampler.unwrap_or(true);
                    let normalizer = normalizer.unwrap_or(true);
                    let transformers = transformers.unwrap_or_else(|| {
                        transformer_ids
                            .iter()
                            .map(|x| x.iter().map(|_| true).collect())
                            .collect()
                    });

                    if transformers
                        .iter()
                        .zip(transformer_ids.iter())
                        .any(|(a, b)| a.len() != b.len())
                    {
                        return Err(Error::msg("The shape of transformers and ids must match!"));
                    }
                    Ok(Self {
                        transformers,
                        sampler,
                        normalizer,
                    })
                }
                _ => Err(Error::msg("Must be a bool or an object!")),
            }
        } else {
            Ok(Self::all_bool(transformer_ids, true))
        }
    }
}
