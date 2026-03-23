use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::backend_options::SecureShellAuth;

#[derive(Clone, Debug, Deserialize, Serialize)]
// Forwarding direction
pub enum TunnelDirection {
    /// From tunnel server to the client (bind: remote server)
    Reverse,
    /// From the client to the server (bind: local machine)
    Forward,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Used by the backend to connect to the remote server.
pub struct Remote {
    /// IP address of the remote server.
    pub ip: IpAddr,
    /// Port of the remote server.
    pub port: i16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// IP and Port of remote or local bind or forwarding point
pub struct IpPort {
    pub ip: IpAddr,
    pub port: i16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Authentication configs, also known as Backends for a tunnel.
pub enum BackendConfig {
    File(String),
    SecureShell(Remote, SecureShellAuth),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Tunnel Manager configuration, defines auto-restarting policy and auto-starting policy.
pub struct TunnelMgr {
    /// If true, Daemon will automatically start the tunnel on Daemon start up.
    pub auto_start: Option<bool>,
    /// If the backend has crashed, the Daemon will restart the tunnel automatically.
    pub auto_restart: Option<bool>,
    /// Interval to wait to restart the tunnel (default: 1000ms).
    pub auto_restart_interval: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TunnelConfig {
    /// Tunnel Manager configuration, defines auto-restarting policy and auto-starting policy.
    pub mgr: Option<TunnelMgr>,
    /// Backend configuration
    pub backend: BackendConfig,
    /// Forwarding direction
    pub direction: TunnelDirection,
    /// Local IpPort
    pub local: IpPort,
    /// Remote IpPort
    pub remote: IpPort,
}
