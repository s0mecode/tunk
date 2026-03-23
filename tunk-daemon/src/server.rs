use std::{error::Error, sync::Arc};
use tokio::net::UnixListener;
use tracing::info;
use tunk_common::daemon_state::DaemonState;

use crate::handler;

pub async fn listen_sock(state: Arc<DaemonState>, sock_path: &str) -> Result<(), Box<dyn Error>> {
    let listener = UnixListener::bind(sock_path)?;

    std::fs::set_permissions(
        sock_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o660),
    )?;

    info!("Daemon listening on {}", sock_path);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handler::handle_connection(state.clone(), stream));
    }
}
