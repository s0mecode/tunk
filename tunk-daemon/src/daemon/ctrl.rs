use std::{sync::Arc, time::Duration};

use tokio::time;
use tracing::{error, info, warn};
use tunk_common::{
    daemon_config::BackendConfig,
    daemon_state::{DaemonState, TunnelStatus},
    error::DaemonError,
};

use crate::{daemon::mgr::spawn_tunnel_manager, loader};

// === Graceful Shutdown ===
pub async fn shutdown_daemon(state: Arc<DaemonState>) {
    info!("Shutting down daemon...");

    let _ = state.shutdown_tx.send(true);

    // Collect keys first, then remove
    let keys: Vec<_> = state
        .tunnel_handles
        .iter()
        .map(|r| r.key().clone())
        .collect();

    for key in keys {
        if let Some((_, tunnel_handle)) = state.tunnel_handles.remove(&key) {
            match time::timeout(Duration::from_secs(5), tunnel_handle.handle).await {
                Ok(Ok(())) => info!("Tunnel '{}' stopped gracefully", key),
                Ok(Err(e)) => error!("Tunnel '{}' task panicked: {}", key, e),
                Err(_) => {
                    error!("Tunnel '{}' did not stop in time, aborting", key);
                    tunnel_handle.abort.abort();
                }
            }
        }
    }

    info!("Daemon shutdown complete");
}

// === Restart Tunnel by Name ===
pub async fn restart_tunnel(state: Arc<DaemonState>, name: &str) -> Result<(), DaemonError> {
    // Get config - tunnel must exist
    let config = state
        .tunnels
        .get(name)
        .map(|r| r.clone())
        .ok_or_else(|| DaemonError::TunnelNotFound(name.to_string()))?;

    // Stop existing tunnel if running (no error if not running)
    if let Some((_, tunnel_handle)) = state.tunnel_handles.remove(name) {
        tunnel_handle.abort.abort();
        let _ = time::timeout(Duration::from_millis(500), tunnel_handle.handle).await;
        info!("Tunnel '{}' stopped for restart", name);
    }

    // Spawn new tunnel manager - regardless of auto_start setting
    spawn_tunnel_manager(state.clone(), name.to_string(), Arc::new(config));

    info!("Tunnel '{}' restarted", name);
    Ok(())
}

// === Start/Stop/Status ===

pub async fn start_tunnel(state: Arc<DaemonState>, name: &str) -> Result<(), DaemonError> {
    if state.tunnel_handles.contains_key(name) {
        return Err(DaemonError::TunnelAlreadyRunning(name.to_string()));
    }

    let config = state
        .tunnels
        .get(name)
        .map(|r| r.clone())
        .ok_or_else(|| DaemonError::TunnelNotFound(name.to_string()))?;

    spawn_tunnel_manager(state.clone(), name.to_string(), Arc::new(config));

    info!("Tunnel '{}' started", name);
    Ok(())
}

pub async fn stop_tunnel(state: Arc<DaemonState>, name: &str) -> Result<(), DaemonError> {
    let (_, tunnel_handle) = state
        .tunnel_handles
        .remove(name)
        .ok_or_else(|| DaemonError::TunnelNotFound(name.to_string()))?;

    tunnel_handle.abort.abort();

    match time::timeout(Duration::from_secs(3), tunnel_handle.handle).await {
        Ok(Ok(())) => {
            info!("Tunnel '{}' stopped", name);
            Ok(())
        }
        Ok(Err(e)) if e.is_cancelled() => {
            info!("Tunnel '{}' stopped", name);
            Ok(())
        }
        Ok(Err(e)) => Err(DaemonError::TaskPanic(e.to_string())),
        Err(_) => {
            error!("Tunnel '{}' did not stop in time", name);
            Err(DaemonError::Timeout)
        }
    }
}

pub async fn get_tunnel_status(
    state: Arc<DaemonState>,
    name: &str,
) -> Result<TunnelStatus, DaemonError> {
    let config = state
        .tunnels
        .get(name)
        .map(|r| r.clone())
        .ok_or_else(|| DaemonError::TunnelNotFound(name.to_string()))?;

    Ok(TunnelStatus {
        name: name.to_string(),
        running: state.tunnel_handles.contains_key(name),
        config,
    })
}

pub async fn list_tunnels(state: Arc<DaemonState>) -> Result<Vec<TunnelStatus>, DaemonError> {
    let statuses = state
        .tunnels
        .iter()
        .map(|entry| {
            let name = entry.key();
            let config = entry.value();

            TunnelStatus {
                name: name.clone(),
                running: state.tunnel_handles.contains_key(name),
                config: config.clone(),
            }
        })
        .collect();

    Ok(statuses)
}

pub async fn reload_daemon(state: Arc<DaemonState>) -> Result<(), DaemonError> {
    info!("Reloading daemon configuration...");

    let backend_path = state.backend_path.clone();
    let tunnel_path = state.tunnel_path.clone();

    let keys: Vec<_> = state
        .tunnel_handles
        .iter()
        .map(|r| r.key().clone())
        .collect();

    for key in keys {
        if let Some((_, tunnel_handle)) = state.tunnel_handles.remove(&key) {
            tunnel_handle.abort.abort();
            let _ = time::timeout(Duration::from_millis(500), tunnel_handle.handle).await;
        }
    }

    state.tunnels.clear();
    state.backends.clear();

    let backends = loader::load_backends(&backend_path).await?;
    let tunnels = loader::load_tunnels(&tunnel_path).await?;

    info!("Reloaded {} tunnels...", tunnels.len());

    for (name, mut tunnel) in tunnels {
        if let BackendConfig::File(backend_file) = &tunnel.backend
            && let Some(backend) = backends.get(backend_file)
        {
            tunnel.backend = backend.clone();
        }

        if matches!(&tunnel.backend, &BackendConfig::File(_)) {
            warn!("Backend of tunnel '{}' dosen't exist, skipping", name);
            continue;
        }

        state.tunnels.insert(name.clone(), tunnel.clone());

        if let Some(mgr) = &tunnel.mgr
            && matches!(mgr.auto_start, Some(true))
        {
            spawn_tunnel_manager(state.clone(), name.clone(), Arc::new(tunnel));
        }
    }

    for (name, backend) in backends {
        state.backends.insert(name, backend);
    }

    info!("Daemon reload complete");
    Ok(())
}
