use std::sync::Arc;

use anyhow::{Ok, Result};
use axum::{routing::get, Router};
use clap::Parser;
use tokio::runtime::Builder;
use web_rwkv_axum::{
    app::AppState,
    cli::LaunchArgs,
    routes::{hello_world, ws},
    components::{permit::BatchRequest, pipeline::Pipeline, softmax::Softmax},
};

async fn app(args: LaunchArgs) -> Result<()> {
    let model_config = args.get_config()?;

    let context = model_config.model.create_context().await?;
    let model = Arc::new(model_config.model.load_model(&context).await?);
    let softmax = Softmax::new(model.clone(), model_config.model.get_batch_size()).await;
    let batch_lock = BatchRequest::new();

    let (softmax_sender, softmax_handle) = softmax.run().await;
    let (infer_sender, model_handle) = Pipeline::start(
        model_config.model.get_batch_size(),
        context.clone(),
        model.clone(),
        batch_lock.clone(),
    )
    .await;

    let shared_state = AppState::new(
        &model_config,
        infer_sender.clone(),
        softmax_sender.clone(),
        context.clone(),
        model.clone(),
        batch_lock.clone(),
    )
    .await?;

    let app = Router::new()
        .route("/", get(hello_world::handler))
        .route("/ws", get(ws::handler))
        .with_state(shared_state);

    axum::Server::bind(&args.get_addr_port()?)
        .serve(app.into_make_service())
        .await?;

    drop(infer_sender);
    drop(softmax_sender);
    model_handle.await?;
    softmax_handle.await?;
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
