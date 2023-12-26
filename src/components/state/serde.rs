use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::*;
use web_rwkv::model::{v4, v5};
use web_rwkv::tensor::shape::Shape;

use crate::components::model::AxumBackedState;

#[derive(Deserialize, Serialize)]
struct StateV4 {
    shape: Shape,
    data: Vec<f32>,
}

#[derive(Deserialize, Serialize)]
struct StateV5 {
    max_batch: usize,
    chunk_size: usize,
    head_size: usize,
    data: Vec<(Shape, Vec<f32>)>,
}

#[derive(Deserialize, Serialize)]
enum AxumBackedStateRepr {
    V4(StateV4),
    V5(StateV5),
}

impl AxumBackedStateRepr {
    pub fn new(state: &AxumBackedState) -> Self {
        match state {
            AxumBackedState::V4(state) => Self::V4(StateV4 {
                shape: state.shape,
                data: state.data.to_vec(),
            }),
            AxumBackedState::V5(state) => Self::V5(StateV5 {
                max_batch: state.max_batch,
                chunk_size: state.chunk_size,
                head_size: state.head_size,
                data: state.data.to_vec(),
            }),
        }
    }

    pub fn into_state(self) -> AxumBackedState {
        match self {
            AxumBackedStateRepr::V4(StateV4 { shape, data }) => {
                AxumBackedState::V4(v4::BackedState {
                    shape,
                    data: data.into(),
                })
            }
            AxumBackedStateRepr::V5(StateV5 {
                data,
                max_batch,
                chunk_size,
                head_size,
            }) => AxumBackedState::V5(v5::BackedState {
                max_batch,
                chunk_size,
                head_size,
                data: data.into(),
            }),
        }
    }
}

pub async fn dump_state(state: &AxumBackedState, path: PathBuf) -> Result<()> {
    let repr = AxumBackedStateRepr::new(state);
    let mut file = File::create(path).await?;
    file.write_all(&serde_cbor::to_vec(&repr)?).await?;
    Ok(())
}

pub async fn load_state(path: PathBuf) -> Result<AxumBackedState> {
    let mut buf = Vec::with_capacity(1024 * 1024 * 16);
    File::open(path).await?.read_to_end(&mut buf).await?;
    Ok((serde_cbor::from_slice::<AxumBackedStateRepr>(&buf)?).into_state())
}
