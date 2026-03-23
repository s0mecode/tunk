mod cli;

use clap::Parser;
use cli::Cli;
use protobuilder::{LengthPrefix, Protocol};
use tokio::net::UnixStream;
use tunk_common::rpc::{CliMessage, DaemonMessage};

const DEFAULT_SOCK_PATH: &str = "/tmp/tunk.sock";

struct Output;

impl Output {
    fn success(&self, msg: &str) {
        println!("\x1b[92m✓\x1b[0m {}", msg);
    }

    fn error(&self, msg: &str) {
        eprintln!("\x1b[91m✗\x1b[0m {}", msg);
    }

    fn info(&self, msg: &str) {
        println!("  {}", msg);
    }

    fn header(&self, msg: &str) {
        println!("\x1b[1;36m{}\x1b[0m", msg);
    }

    fn tunnel_status(&self, name: &str, running: bool) {
        let status = if running {
            "\x1b[92mrunning\x1b[0m"
        } else {
            "\x1b[90mstopped\x1b[0m"
        };
        println!("  {}  {}", name, status);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let out = Output;

    let sock_path =
        std::env::var("TUNK_SOCK_PATH").unwrap_or_else(|_| DEFAULT_SOCK_PATH.to_string());

    let stream = UnixStream::connect(&sock_path).await?;
    let mut proto: Protocol<_, CliMessage, DaemonMessage, _> =
        Protocol::<_, CliMessage, DaemonMessage, _>::builder()
            .framing(LengthPrefix::u32())
            .build(stream)?;

    match cli.command {
        cli::Commands::Start { name } => {
            proto.send(CliMessage::StartTunnel { name }).await?;
            match proto.recv().await? {
                DaemonMessage::StartTunnelResponse { ok, error } => {
                    if ok {
                        out.success("Tunnel started");
                    } else {
                        out.error(&format!("{}", error.unwrap()));
                    }
                }
                _ => out.error("Unexpected response from daemon"),
            }
        }
        cli::Commands::Stop { name } => {
            proto.send(CliMessage::StopTunnel { name }).await?;
            match proto.recv().await? {
                DaemonMessage::StopTunnelResponse { ok, error } => {
                    if ok {
                        out.success("Tunnel stopped");
                    } else {
                        out.error(&format!("{}", error.unwrap()));
                    }
                }
                _ => out.error("Unexpected response from daemon"),
            }
        }
        cli::Commands::Restart { name } => {
            proto.send(CliMessage::RestartTunnel { name }).await?;
            match proto.recv().await? {
                DaemonMessage::RestartTunnelResponse { ok, error } => {
                    if ok {
                        out.success("Tunnel restarted");
                    } else {
                        out.error(&format!("{}", error.unwrap()));
                    }
                }
                _ => out.error("Unexpected response from daemon"),
            }
        }
        cli::Commands::Status { name } => {
            proto.send(CliMessage::GetTunnelStatus { name }).await?;
            match proto.recv().await? {
                DaemonMessage::GetTunnelStatusResponse { ok, status, error } => {
                    if ok {
                        if let Some(s) = status {
                            out.header("Tunnel Status");
                            out.info(&format!("Name:    {}", s.name));
                            out.info(&format!(
                                "Running: {}",
                                if s.running { "yes" } else { "no" }
                            ));
                        }
                    } else {
                        out.error(&format!("{}", error.unwrap()));
                    }
                }
                _ => out.error("Unexpected response from daemon"),
            }
        }
        cli::Commands::List => {
            proto.send(CliMessage::ListTunnels).await?;
            match proto.recv().await? {
                DaemonMessage::ListTunnelsResponse { ok, tunnels, error } => {
                    if ok {
                        out.header("Configured Tunnels");
                        if tunnels.is_empty() {
                            out.info("No tunnels configured");
                        } else {
                            for tunnel in tunnels {
                                out.tunnel_status(&tunnel.name, tunnel.running);
                            }
                        }
                    } else {
                        out.error(&format!("{}", error.unwrap()));
                    }
                }
                _ => out.error("Unexpected response from daemon"),
            }
        }
        cli::Commands::Reload => {
            proto.send(CliMessage::Reload).await?;
            match proto.recv().await? {
                DaemonMessage::ReloadResponse { ok, error } => {
                    if ok {
                        out.success("Daemon configuration reloaded");
                    } else {
                        out.error(&format!("{}", error.unwrap()));
                    }
                }
                _ => out.error("Unexpected response from daemon"),
            }
        }
    }

    Ok(())
}
