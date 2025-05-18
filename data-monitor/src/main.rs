mod app_data;
mod config;
mod model;
mod router;
mod timed_task;

use salvo::prelude::*;
use timed_task::register_timed_task;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = std::env::args().collect::<Vec<_>>();

    let config = config::Config::from_path(opt.get(1).unwrap_or(&"config.toml".into()))
        .expect("Failed to load config");
    let _g = ai_flow_synth::utils::enable_log(&config.log_config).unwrap();
    let app_data = app_data::AppData::new(&config).await;

    register_timed_task(app_data.clone()).await;

    let router = Router::new()
        .hoop(affix_state::inject(app_data))
        .push(router::create_router());

    let address = "127.0.0.1:7878";
    let acceptor = TcpListener::new(address).bind().await;
    Server::new(acceptor).serve(router).await;
    info!("Server started on {}", address);

    Ok(())
}
