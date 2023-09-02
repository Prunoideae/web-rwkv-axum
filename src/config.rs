use std::path::PathBuf;

use anyhow::{Ok, Result};
use memmap2::Mmap;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

use serde::Deserialize;
use web_rwkv::{
    context::{Context, ContextBuilder, Instance},
    model::{LayerFlags, Model, ModelBuilder, Quantization},
    tokenizer::Tokenizer,
    wgpu::Adapter,
};

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

#[derive(Debug, Deserialize, Clone)]
pub struct ModelSpec {
    path: PathBuf,
    #[serde(default)]
    max_batch_count: props::BatchSize,
    #[serde(default)]
    max_chunk_count: props::ChunkSize,
    preference: Option<props::Preference>,
    adapter: Option<usize>,
    quantization: Option<u64>,
}

impl ModelSpec {
    pub fn get_batch_size(&self) -> usize {
        self.max_batch_count.get()
    }

    pub fn get_chunk_size(&self) -> usize {
        self.max_chunk_count.get()
    }

    pub async fn select_adapter(&self, instance: &Instance) -> Result<Adapter> {
        if let Some(preference) = &self.preference {
            Ok(instance.adapter(preference.to_web_rwkv()).await?)
        } else if let Some(index) = self.adapter {
            Ok(instance.select_adapter(index)?)
        } else {
            Ok(instance.select_adapter(0)?)
        }
    }

    pub async fn create_context(&self) -> Result<Context> {
        let adapter = self.select_adapter(&Instance::new()).await?;
        Ok(ContextBuilder::new(adapter)
            .with_default_pipelines()
            .with_quant_pipelines()
            .build()
            .await?)
    }

    pub async fn load_model<'a>(&self, context: &'a Context) -> Result<Model<'a, '_>> {
        let file = File::open(&self.path).await?;
        let map = unsafe { Mmap::map(&file)? };
        let quant = self
            .quantization
            .map(|bits| Quantization::Int8(LayerFlags::from_bits_retain(bits)))
            .unwrap_or_default();

        Ok(ModelBuilder::new(context, &map).with_quant(quant).build()?)
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
pub struct ModelConfig {
    pub model: ModelSpec,
    pub tokenizer: TokenizerSpec,
}
