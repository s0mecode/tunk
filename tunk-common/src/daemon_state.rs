use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::{
    sync::watch,
    task::{AbortHandle, JoinHandle},
};

use crate::daemon_config::{BackendConfig, TunnelConfig};

pub struct DaemonState {
    pub tunnels: DashMap<String, TunnelConfig>,
    pub backends: DashMap<String, BackendConfig>,
    pub tunnel_handles: DashMap<String, TunnelHandle>,
    pub shutdown_tx: watch::Sender<bool>,
    pub backend_path: PathBuf,
    pub tunnel_path: PathBuf,
}

pub struct TunnelHandle {
    pub handle: JoinHandle<()>,
    pub abort: AbortHandle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub name: String,
    pub running: bool,
    pub config: TunnelConfig,
}

impl DaemonState {
    pub fn new(backend_path: PathBuf, tunnel_path: PathBuf) -> Self {
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            tunnels: DashMap::new(),
            backends: DashMap::new(),
            tunnel_handles: DashMap::new(),
            shutdown_tx,
            backend_path,
            tunnel_path,
        }
    }

    pub fn shutdown_tx(&self) -> watch::Sender<bool> {
        self.shutdown_tx.clone()
    }
}
