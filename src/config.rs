use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use futures_util::{stream, StreamExt};

use memmap2::Mmap;

use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

use serde::Deserialize;
use web_rwkv::{
    context::{Context, ContextBuilder, Instance},
    model::{
        loader::Loader,
        Lora, LoraBlend, LoraBlendPattern, ModelBuilder, ModelInfo,
        ModelVersion::{V4, V5},
        Quant,
    },
    tokenizer::Tokenizer,
    wgpu::{Adapter, Backends},
};

use crate::components::model::AxumModel;

mod props {
    use serde::Deserialize;
    use web_rwkv::wgpu::PowerPreference;

    #[derive(Debug, Deserialize, Clone)]
    pub struct BatchSize(usize);
    impl Default for BatchSize {
        fn default() -> Self {
            BatchSize(32)
        }
    }

    impl BatchSize {
        pub fn get(&self) -> usize {
            self.0
        }
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct ChunkSize(usize);
    impl Default for ChunkSize {
        fn default() -> Self {
            ChunkSize(256)
        }
    }

    impl ChunkSize {
        pub fn get(&self) -> usize {
            self.0
        }
    }

    #[derive(Debug, Deserialize, Clone)]
    pub enum Preference {
        HighPerformance = 0,
        LowPower = 1,
    }

    impl Preference {
        pub fn to_web_rwkv(&self) -> PowerPreference {
            match self {
                Self::HighPerformance => PowerPreference::HighPerformance,
                Self::LowPower => PowerPreference::LowPower,
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct BlendConfig {
    pattern: String,
    alpha: f32,
}

#[derive(Deserialize, Debug, Clone)]
struct LoraConfig {
    path: PathBuf,
    blends: Vec<BlendConfig>,
}

impl LoraConfig {
    async fn load(&self) -> Result<Lora> {
        let mut data = Vec::new();
        File::open(&self.path).await?.read_to_end(&mut data).await?;
        Ok(Lora {
            data,
            blend: LoraBlend(
                self.blends
                    .iter()
                    .map(|BlendConfig { pattern, alpha }| LoraBlendPattern::new(pattern, *alpha))
                    .collect::<Result<_>>()?,
            ),
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelSpec {
    path: PathBuf,
    #[serde(default)]
    max_batch_count: props::BatchSize,
    #[serde(default)]
    max_chunk_count: props::ChunkSize,
    max_state_size: Option<usize>,
    max_concurrency: Option<usize>,
    preference: Option<props::Preference>,
    adapter: Option<usize>,
    quantization: Option<String>,
    max_infer_tokens: Option<usize>,
    lora_config: Option<Vec<LoraConfig>>,
}

impl ModelSpec {
    pub fn get_batch_size(&self) -> usize {
        self.max_batch_count.get()
    }

    pub fn get_chunk_size(&self) -> usize {
        self.max_chunk_count.get()
    }

    pub fn get_max_concurrency(&self) -> usize {
        self.max_concurrency.unwrap_or(8)
    }

    pub fn get_max_state_size(&self) -> Option<usize> {
        self.max_state_size
    }

    pub fn get_max_infer_tokens(&self) -> usize {
        self.max_infer_tokens.unwrap_or(256)
    }

    pub async fn select_adapter(&self, instance: &Instance) -> Result<Adapter> {
        if let Some(preference) = &self.preference {
            Ok(instance.adapter(preference.to_web_rwkv()).await?)
        } else if let Some(index) = self.adapter {
            Ok(instance.select_adapter(Backends::PRIMARY, index)?)
        } else {
            Ok(instance.select_adapter(Backends::PRIMARY, 0)?)
        }
    }

    pub async fn create_context(&self) -> Result<Context> {
        let adapter = self.select_adapter(&Instance::new()).await?;
        println!("{:?}", adapter.get_info());
        let context = ContextBuilder::new(adapter).with_default_pipelines();
        Ok(context.build().await?)
    }

    fn parse_quant(quant: Option<String>, model_info: &ModelInfo) -> HashMap<usize, Quant> {
        let mut quants = HashMap::new();

        for layer in 0..model_info.num_layer {
            quants.insert(layer, Quant::None);
        }

        if let Some(quant) = quant {
            for (index, quant) in quant.chars().map(|x| x.to_string().parse()).enumerate() {
                quants.insert(
                    index,
                    match quant {
                        Ok(1) => Quant::Int8,
                        Ok(2) => Quant::NF4,
                        _ => Quant::None,
                    },
                );
            }
        }
        quants
    }

    pub async fn load_model(&self, context: &Context) -> Result<AxumModel> {
        let file = File::open(&self.path).await?;
        let map = unsafe { Mmap::map(&file)? };
        let info = Loader::info(&map)?;

        let quants = ModelSpec::parse_quant(self.quantization.clone(), &info);
        let loras = stream::iter(self.lora_config.clone().unwrap_or(Vec::new()))
            .then(|x| async move { x.load().await })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(match info.version {
            V4 => AxumModel::V4({
                let mut builder = ModelBuilder::new(context, &map)
                    .with_token_chunk_size(self.get_chunk_size())
                    .with_head_chunk_size(8192)
                    .with_quant(quants)
                    .with_turbo(true);
                for lora in loras {
                    builder = builder.add_lora(lora)
                }
                builder.build()?
            }),
            V5 => AxumModel::V5({
                let mut builder = ModelBuilder::new(context, &map)
                    .with_token_chunk_size(self.get_chunk_size())
                    .with_head_chunk_size(8192)
                    .with_quant(quants)
                    .with_turbo(true);
                for lora in loras {
                    builder = builder.add_lora(lora)
                }
                builder.build()?
            }),
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TokenizerSpec {
    path: PathBuf,
}

impl TokenizerSpec {
    pub async fn load_tokenizer(&self) -> Result<Tokenizer> {
        let content = {
            let mut reader = BufReader::with_capacity(1024 * 128, File::open(&self.path).await?);
            let mut buf = String::new();
            reader.read_to_string(&mut buf).await?;
            buf
        };
        Ok(Tokenizer::new(&content)?)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AxumSpec {
    pub state_dump: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelConfig {
    pub model: ModelSpec,
    pub tokenizer: TokenizerSpec,
    pub axum: AxumSpec,
}
