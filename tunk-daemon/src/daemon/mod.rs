pub mod ctrl;
pub mod mgr;
pub mod utils;

use tunk_common::{daemon_config::BackendConfig, daemon_state::DaemonState, error::DaemonError};

use std::{path::PathBuf, sync::Arc};
use tracing::{info, warn};

use crate::{daemon::mgr::spawn_tunnel_manager, loader};

// === Init ===
pub async fn init_daemon(
    state: Arc<DaemonState>,
    backend_path: PathBuf,
    tunnel_path: PathBuf,
) -> Result<(), DaemonError> {
    let backends = loader::load_backends(&backend_path).await?;
    let tunnels = loader::load_tunnels(&tunnel_path).await?;

    info!("Initializing {} tunnels...", tunnels.len());

    for (name, mut tunnel) in tunnels {
        // Resolve backend from file if not inline
        if let BackendConfig::File(backend_file) = &tunnel.backend
            && let Some(backend) = backends.get(backend_file)
        {
            tunnel.backend = backend.clone();
        }

        if matches!(&tunnel.backend, &BackendConfig::File(_)) {
            warn!("Backend of tunnel '{}' dosen't exist, skipping", name);
            continue;
        }

        // Insert tunnel into state
        state.tunnels.insert(name.clone(), tunnel.clone());

        // Start tunnel if auto_start enabled
        if let Some(mgr) = &tunnel.mgr
            && matches!(mgr.auto_start, Some(true))
        {
            spawn_tunnel_manager(state.clone(), name.clone(), Arc::new(tunnel));
        }
    }

    // Insert backends
    for (name, backend) in backends {
        state.backends.insert(name, backend);
    }

    Ok(())
}
