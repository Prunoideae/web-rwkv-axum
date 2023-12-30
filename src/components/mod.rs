use std::collections::HashMap;

use anyhow::{Error, Result};
use serde_json::Value;

use crate::{app::AppState, hashmap_ex};

use self::{
    normalizer::{types::Normalizer, classifier_free_guidance},
    sampler::{nucleus, types::Sampler, typical},
    terminal::{lengthed, types::Terminal, until},
    transformer::{
        bnf_constraint, disable_tokens, global_penalty, sliding_penalty, types::Transformer,logits_compressor,
    },
};

pub mod infer;
pub mod model;
pub mod normalizer;
pub mod pipeline;
pub mod sampler;
pub mod softmax;
pub mod state;
pub mod terminal;
pub mod transformer;

pub enum InferenceInterruption {
    Exhaustion,
    Error(Error),
}

impl InferenceInterruption {
    pub fn exhausted(&self) -> bool {
        match self {
            InferenceInterruption::Exhaustion => true,
            InferenceInterruption::Error(_) => false,
        }
    }
}

pub struct Registry {
    terminal: HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Terminal>>>,
    transformer: HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Transformer>>>,
    sampler: HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Sampler>>>,
    normalizer: HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Normalizer>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            terminal: hashmap_ex! {
                HashMap<&str, fn(AppState, Option<Value>) -> Result<Box<dyn Terminal>>>,
                    {
                        "lengthed" => lengthed::initialize_lenghted,
                        "until" => until::intialize_until,
                    }
            },
            transformer: hashmap_ex! {
                HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Transformer>>>,
                    {
                        "global_penalty" => global_penalty::initialize_global,
                        "sliding_penalty" => sliding_penalty::initialize_sliding,
                        "disable_token" => disable_tokens::initialize_disable,
                        "bnf_grammar" => bnf_constraint::BNFConstraint::initialize,
                        "logits_compressor"=>logits_compressor::LogitsCompressor::initialize
                    }
            },
            sampler: hashmap_ex! {
                HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Sampler>>>,
                    {
                        "nucleus" => nucleus::initialize,
                        "typical" => typical::TypicalSampler::initialize
                    }
            },
            normalizer: hashmap_ex! {
                HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Normalizer>>>,
                    {
                        "classifier_free_guidance" => classifier_free_guidance::ClassifierFreeGuidance::initialize,
                    }
            },
        }
    }

    pub fn create_terminal(
        &self,
        key: &str,
        state: AppState,
        data: Option<Value>,
    ) -> Result<Box<dyn Terminal>> {
        let constructor = self.terminal.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Terminal not found!"))
        }
    }

    pub fn create_sampler(
        &self,
        key: &str,
        state: AppState,
        data: Option<Value>,
    ) -> Result<Box<dyn Sampler>> {
        let constructor = self.sampler.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Sampler not found!"))
        }
    }

    pub fn create_transformer(
        &self,
        key: &str,
        state: AppState,
        data: Option<Value>,
    ) -> Result<Box<dyn Transformer>> {
        let constructor = self.transformer.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Transformer not found!"))
        }
    }

    pub fn create_normalizer(
        &self,
        key: &str,
        state: AppState,
        data: Option<Value>,
    ) -> Result<Box<dyn Normalizer>> {
        let constructor = self.normalizer.get(key);
        if let Some(constructor) = constructor {
            Ok(constructor(state, data)?)
        } else {
            Err(Error::msg("Normalizer not found!"))
        }
    }
}
