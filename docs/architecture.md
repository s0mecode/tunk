# Architecture

## Overview

tunk follows a client-daemon architecture for tunnel management:

```
┌─────────────────────────────────────────────────────────────┐
│                        tunk (CLI)                           │
│                     Unix Stream Client                      │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            │
┌───────────────────────────▼──────────────────────────────────┐
│                     tunkd (Daemon)                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                   Server (server.rs)                   │  │
│  │              Unix Stream Listener + Handler            │  │
│  └────────────────────────┬───────────────────────────────┘  │
│                           │                                  │
│  ┌────────────────────────▼───────────────────────────────┐  │
│  │              Handler (handler.rs)                      │  │
│  │         Routes CliMessage → Daemon Functions           │  │
│  └────────────────────────┬───────────────────────────────┘  │
│                           │                                  │
│  ┌────────────────────────▼───────────────────────────────┐  │
│  │                 Controller (ctrl.rs)                   │  │
│  │                                                        │  │
│  │         start/stop/restart/reload/list/status          │  │
│  └────────────────────────┬───────────────────────────────┘  │
│                           │                                  │
│  ┌────────────────────────▼───────────────────────────────┐  │
│  │                   Daemon State                         │  │
│  │  • tunnels: DashMap<String, TunnelConfig>              │  │
│  │  • backends: DashMap<String, BackendConfig>            │  │
│  │  • tunnel_handles: DashMap<String, TunnelHandle>       │  │
│  │  • shutdown_tx: watch::Sender<bool>                    │  │
│  └─────────────────────────────────────────────────────────┘ │
│                           │                                  │
│  ┌────────────────────────▼───────────────────────────────┐  │
│  │              Tunnel Manager (mgr.rs)                   │  │
│  │    spawns backend + monitors health + auto-restart     │  │
│  └────────────────────────┬───────────────────────────────┘  │
│                           │                                  │
│  ┌────────────────────────▼───────────────────────────────┐  │
│  │              Backends (tunk-backends/)                 │  │
│  │        SecureShell ──► ssh process (-L/-R)             │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Components

### tunk-cli

Command-line client that:
- Parses user commands via `clap`
- Connects to daemon via Unix socket
- Sends `CliMessage` requests
- Receives and displays `DaemonMessage` responses

Uses `protobuilder` with length-prefixed framing for protocol.

### tunkd (Daemon)

The main daemon process that:
- Loads configurations on startup
- Listens on Unix socket for CLI commands
- Manages tunnel lifecycle
- Handles graceful shutdown

**Modules:**
- `server.rs`: Unix socket listener
- `handler.rs`: Connection handler, routes messages
- `daemon/mod.rs`: Initialization logic
- `daemon/ctrl.rs`: Control operations (start/stop/restart/reload)
- `daemon/mgr.rs`: Tunnel manager - spawns backends, monitors health
- `loader.rs`: Loads RON configs from disk

### tunk-common

Shared types between CLI and daemon:
- `rpc.rs`: `CliMessage` / `DaemonMessage` enums
- `daemon_config.rs`: `TunnelConfig`, `BackendConfig`, `TunnelDirection`
- `daemon_state.rs`: `DaemonState`, `TunnelHandle`, `TunnelStatus`
- `error.rs`: `DaemonError`, `BackendError`

### tunk-backends

Pluggable backend implementations:

**BackendController trait:**
```rust
#[async_trait]
pub trait BackendController: Send + Sync {
    async fn disconnect(&mut self) -> Result<(), BackendError>;
    async fn is_alive(&self) -> bool;
}
```

**SecureShell backend:**
- Spawns `ssh` process with `-L` (Forward) or `-R` (Reverse)
- Uses `tokio::process::Command`
- Monitors child process via `is_alive()`

## RPC Protocol

Communication via Unix Domain Socket with length-prefixed framing:

```rust
Protocol::<_, CliMessage, DaemonMessage, _>::builder()
    .framing(LengthPrefix::u32())
    .build(stream)
```

### Messages

**Client → Daemon:**
```rust
enum CliMessage {
    StartTunnel { name: String },
    StopTunnel { name: String },
    RestartTunnel { name: String },
    GetTunnelStatus { name: String },
    ListTunnels,
    Reload,
}
```

**Daemon → Client:**
```rust
enum DaemonMessage {
    StartTunnelResponse { ok: bool, error: Option<DaemonError> },
    StopTunnelResponse { ok: bool, error: Option<DaemonError> },
    // ... etc
}
```

## Data Flow

### Start Tunnel
```
CLI: tunk start web
  → CliMessage::StartTunnel { name: "web" }
  → Unix socket

Daemon receives:
  → handler::handle_connection
  → ctrl::start_tunnel
    → validates tunnel exists
    → spawn_tunnel_manager
      → creates TunnelHandle (JoinHandle + AbortHandle)
      → inserts into tunnel_handles DashMap
      → spawns async task: tunnel_manager()

tunnel_manager():
  → create_backend() → SecureShellConnection::connect()
    → builds ssh command with -R/-L flags
    → spawns ssh child process
  → loop: is_alive() check every 1s
    → if dead: reconnect with backoff
  → on shutdown signal: disconnect() → kill ssh
```

### Configuration Loading

```
Daemon startup:
  → init_daemon()
  → loader::load_backends(backend_path)
    → read all *.ron files in backends/
    → parse as BackendConfig
    → store in state.backends
  → loader::load_tunnels(tunnel_path)
    → read all *.ron files in tunnels/
    → parse as TunnelConfig
    → resolve BackendConfig::File references to actual backends
    → insert into state.tunnels
    → if auto_start: spawn_tunnel_manager()
```

## Thread Safety

- `DaemonState` fields use `DashMap` (concurrent HashMap)
- `BackendController` trait requires `Send + Sync`
- `SecureShellConnection` uses `Arc<Mutex<SecureShellInner>>`

## Shutdown Sequence

```
1. ctrl_c or SIGINT received
2. shutdown_daemon() called
3. shutdown_tx.send(true) - broadcast to all tunnel managers
4. For each tunnel:
   - Abort handle
   - Wait up to 5s for graceful exit
   - Force abort if still running
5. Daemon exits
```
