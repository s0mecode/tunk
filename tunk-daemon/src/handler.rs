use protobuilder::{LengthPrefix, Protocol};
use std::{error::Error, sync::Arc};
use tokio::net::UnixStream;
use tunk_common::{
    daemon_state::DaemonState,
    rpc::{CliMessage, DaemonMessage},
};

use crate::daemon::ctrl::{
    get_tunnel_status, list_tunnels, reload_daemon, restart_tunnel, start_tunnel, stop_tunnel,
};

pub async fn handle_connection(
    state: Arc<DaemonState>,
    stream: UnixStream,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut proto = Protocol::<_, DaemonMessage, CliMessage, _>::builder()
        .framing(LengthPrefix::u32())
        .build(stream)?;

    loop {
        match proto.recv().await? {
            CliMessage::StartTunnel { name } => match start_tunnel(state.clone(), &name).await {
                Ok(()) => {
                    proto
                        .send(DaemonMessage::StartTunnelResponse {
                            ok: true,
                            error: None,
                        })
                        .await?;
                }
                Err(e) => {
                    proto
                        .send(DaemonMessage::StartTunnelResponse {
                            ok: false,
                            error: Some(e),
                        })
                        .await?;
                }
            },
            CliMessage::StopTunnel { name } => match stop_tunnel(state.clone(), &name).await {
                Ok(()) => {
                    proto
                        .send(DaemonMessage::StopTunnelResponse {
                            ok: true,
                            error: None,
                        })
                        .await?;
                }
                Err(e) => {
                    proto
                        .send(DaemonMessage::StopTunnelResponse {
                            ok: false,
                            error: Some(e),
                        })
                        .await?;
                }
            },
            CliMessage::RestartTunnel { name } => {
                match restart_tunnel(state.clone(), &name).await {
                    Ok(()) => {
                        proto
                            .send(DaemonMessage::RestartTunnelResponse {
                                ok: true,
                                error: None,
                            })
                            .await?;
                    }
                    Err(e) => {
                        proto
                            .send(DaemonMessage::RestartTunnelResponse {
                                ok: false,
                                error: Some(e),
                            })
                            .await?;
                    }
                }
            }
            CliMessage::GetTunnelStatus { name } => {
                match get_tunnel_status(state.clone(), &name).await {
                    Ok(status) => {
                        proto
                            .send(DaemonMessage::GetTunnelStatusResponse {
                                ok: true,
                                status: Some(status),
                                error: None,
                            })
                            .await?;
                    }
                    Err(e) => {
                        proto
                            .send(DaemonMessage::GetTunnelStatusResponse {
                                ok: false,
                                status: None,
                                error: Some(e),
                            })
                            .await?;
                    }
                }
            }
            CliMessage::ListTunnels => match list_tunnels(state.clone()).await {
                Ok(tunnels) => {
                    proto
                        .send(DaemonMessage::ListTunnelsResponse {
                            ok: true,
                            tunnels,
                            error: None,
                        })
                        .await?;
                }
                Err(e) => {
                    proto
                        .send(DaemonMessage::ListTunnelsResponse {
                            ok: false,
                            tunnels: Vec::new(),
                            error: Some(e),
                        })
                        .await?;
                }
            },
            CliMessage::Reload => match reload_daemon(state.clone()).await {
                Ok(()) => {
                    proto
                        .send(DaemonMessage::ReloadResponse {
                            ok: true,
                            error: None,
                        })
                        .await?;
                }
                Err(e) => {
                    proto
                        .send(DaemonMessage::ReloadResponse {
                            ok: false,
                            error: Some(e),
                        })
                        .await?;
                }
            },
        }
    }

    // Ok(())
}
