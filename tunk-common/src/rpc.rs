use serde::{Deserialize, Serialize};

use crate::{daemon_state::TunnelStatus, error::DaemonError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CliMessage {
    StartTunnel { name: String },
    StopTunnel { name: String },
    RestartTunnel { name: String },
    GetTunnelStatus { name: String },
    ListTunnels,
    Reload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DaemonMessage {
    StartTunnelResponse {
        ok: bool,
        error: Option<DaemonError>,
    },
    StopTunnelResponse {
        ok: bool,
        error: Option<DaemonError>,
    },
    RestartTunnelResponse {
        ok: bool,
        error: Option<DaemonError>,
    },
    GetTunnelStatusResponse {
        ok: bool,
        status: Option<TunnelStatus>,
        error: Option<DaemonError>,
    },
    ListTunnelsResponse {
        ok: bool,
        tunnels: Vec<TunnelStatus>,
        error: Option<DaemonError>,
    },
    ReloadResponse {
        ok: bool,
        error: Option<DaemonError>,
    },
}
