# Installation

## System Requirements

- Linux (primary target)
- systemd (for automatic installation script)
- SSH client binary in PATH

## Install via Script

For systemd-based systems:

```bash
curl -fsSL https://cdn.nilla.run/i/tunk-sd | bash
```

This installs both `tunk` (CLI) and `tunkd` (daemon) binaries to the `~/.local/bin`, and creates a systemd user service. 
> Make sure `~/.local/bin` is in your `$PATH`.

## Manual Installation

1. Build from source:
   ```bash
   cargo build --release
   ```

2. Install binaries:
   ```bash
   cp target/release/tunk ~/.local/bin/   # CLI client
   cp target/release/tunkd ~/.local/bin/  # Daemon
   ```

3. Create directories:
   ```bash
   mkdir -p ~/.config/tunk/{backends,tunnels}
   ```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TUNK_SOCK_PATH` | `/tmp/tunk.sock` | Unix socket path |
| `TUNK_BACKEND_PATH` | `~/.config/tunk/backends` | Backend configs |
| `TUNK_TUNNEL_PATH` | `~/.config/tunk/tunnels` | Tunnel configs |
| `SSH_BINARY` | `ssh` | SSH binary path |

## Running the Daemon

```bash
# Run manually (in background)
tunkd &

# Or with custom paths
TUNK_SOCK_PATH=/var/run/tunk.sock tunkd
```

For production, create a systemd service:

```ini
# /etc/systemd/system/tunkd.service
[Unit]
Description=tunk tunnel manager
After=network.target

[Service]
ExecStart=/usr/local/bin/tunkd
Restart=always

[Install]
WantedBy=multi-user.target
```
