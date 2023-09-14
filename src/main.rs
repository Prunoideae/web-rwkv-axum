use anyhow::{Ok, Result};
use axum::{routing::get, Router};
use clap::Parser;
use tokio::runtime::Builder;
use web_rwkv_axum::{
    app::AppState,
    cli::LaunchArgs,
    routes::{hello_world, ws},
};

async fn app(args: LaunchArgs) -> Result<()> {
    let model_config = args.get_config()?;
    let shared_state = AppState::new(&model_config).await?;

    let app = Router::new()
        .route("/", get(hello_world::handler))
        .route("/ws", get(ws::handler))
        .with_state(shared_state);

    axum::Server::bind(&args.get_addr_port()?)
        .serve(app.into_make_service())
        .await?;

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
