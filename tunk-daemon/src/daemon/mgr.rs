use std::{sync::Arc, time::Duration};

use tokio::{sync::watch, time};

use tracing::{error, info, warn};
use tunk_backends::create_backend;
use tunk_common::{
    backend_options::BackendController,
    daemon_config::TunnelConfig,
    daemon_state::{DaemonState, TunnelHandle},
};

pub fn spawn_tunnel_manager(state: Arc<DaemonState>, name: String, config: Arc<TunnelConfig>) {
    let mut shutdown_rx = state.shutdown_tx.subscribe();
    let state_clone = state.clone(); // Clone BEFORE move
    let name_clone = name.clone();

    let handle = tokio::spawn(async move {
        tunnel_manager(state_clone, name_clone, config, &mut shutdown_rx).await;
    });

    // Now state is still available here
    state.tunnel_handles.insert(
        name,
        TunnelHandle {
            abort: handle.abort_handle(),
            handle,
        },
    );
}

// === Tunnel Manager ===
async fn tunnel_manager(
    state: Arc<DaemonState>,
    name: String,
    config: Arc<TunnelConfig>,
    shutdown_rx: &mut watch::Receiver<bool>,
) {
    let interval = Duration::from_millis(
        config
            .mgr
            .as_ref()
            .and_then(|m| m.auto_restart_interval)
            .unwrap_or(1000),
    );

    let mut controller: Option<Box<dyn BackendController>> = None;

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("Tunnel '{}' received shutdown signal", name);
                    if let Some(mut ctrl) = controller.take() {
                        let _ = ctrl.disconnect().await;
                    }
                    break;
                }
            }

            _ = async {
                // Connect if not connected
                if controller.is_none() {
                    match create_backend((*config).clone()).await {
                        Ok(ctrl) => {
                            info!("Tunnel '{}' connected", name);
                            controller = Some(ctrl);
                        }
                        Err(e) => {
                            error!("Tunnel '{}' failed to connect: {}", name, e);
                            time::sleep(interval).await;
                            return;
                        }
                    }
                }

                // Check health
                if let Some(ctrl) = &mut controller
                    && !ctrl.is_alive().await
                {
                    warn!("Tunnel '{}' died, reconnecting...", name);
                    let _ = ctrl.disconnect().await;
                    controller = None;
                    return;
                }

                time::sleep(interval).await;
            } => {}
        }
    }

    state.tunnel_handles.remove(&name);
}
