mod app;
mod cli;
mod commands;
mod config;
mod helper;
mod macros;
mod routes;
mod states;

use anyhow::{Ok, Result};
use app::{AppState, SharedState};
use axum::{routing::get, Router};
use clap::Parser;
use routes::{hello_world, ws};
use states::pipeline::Pipeline;
use tokio::runtime::Builder;

use crate::cli::LaunchArgs;

async fn app(args: LaunchArgs) -> Result<()> {
    let model_config = args.get_config()?;
    let (infer_sender, model_handle) = Pipeline::start(&model_config).await;

    let shared_state = SharedState::new(AppState::new(&model_config, infer_sender.clone()).await?);

    let app = Router::new()
        .route("/", get(hello_world::handler))
        .route("/ws", get(ws::handler))
        .with_state(shared_state);

    axum::Server::bind(&args.get_addr_port()?)
        .serve(app.into_make_service())
        .await?;

    drop(infer_sender);
    model_handle.await??;
    Ok(())
}

fn main() {
    let parsed = LaunchArgs::parse();

    Builder::new_multi_thread()
        .worker_threads(parsed.get_workers())
        .enable_all()
        .build()
        .unwrap()
        .block_on(app(parsed))
        .unwrap()
}
