pub mod daemon;
pub mod handler;
pub mod loader;
pub mod server;

use std::{error::Error, path::PathBuf, sync::Arc};
use tokio::signal;
use tracing_subscriber::EnvFilter;
use tunk_common::daemon_state::DaemonState;

use crate::daemon::ctrl::shutdown_daemon;

pub const DEFAULT_BACKEND_PATH: &str = ".config/tunk/backends";
pub const DEFAULT_TUNNEL_PATH: &str = ".config/tunk/tunnels";
pub const DEFAULT_SOCK_PATH: &str = "/tmp/tunk.sock";

fn get_env_path(env_var: &str, default: &str) -> PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    std::env::var(env_var)
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(default))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let backend_path = get_env_path("TUNK_BACKEND_PATH", DEFAULT_BACKEND_PATH);
    let tunnel_path = get_env_path("TUNK_TUNNEL_PATH", DEFAULT_TUNNEL_PATH);
    let sock_path =
        std::env::var("TUNK_SOCK_PATH").unwrap_or_else(|_| DEFAULT_SOCK_PATH.to_string());

    std::fs::create_dir_all(&backend_path)?;
    std::fs::create_dir_all(&tunnel_path)?;

    let _ = std::fs::remove_file(&sock_path);

    let state = Arc::new(DaemonState::new(backend_path.clone(), tunnel_path.clone()));

    daemon::init_daemon(state.clone(), backend_path, tunnel_path)
        .await
        .expect("Failed to initialize daemon");

    let rpc_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = server::listen_sock(rpc_state, &sock_path).await {
            tracing::error!("RPC server error: {}", e);
        }
    });

    tokio::select! {
        _ = signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, initiating shutdown...");
            shutdown_daemon(state.clone()).await;
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)) => {}
    };

    Ok(())
}
