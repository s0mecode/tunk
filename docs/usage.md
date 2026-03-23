# Usage

## CLI Commands

```bash
tunk start <name>     # Start a tunnel
tunk stop <name>     # Stop a tunnel
tunk restart <name>  # Restart a tunnel
tunk status <name>   # Show tunnel status
tunk list           # List all tunnels
tunk reload         # Reload configurations from disk
```

## Starting a Tunnel

```bash
tunk start web
# ✓ Tunnel started
```

The tunnel starts running in the daemon process. The daemon handles:
- Spawning the SSH process
- Monitoring its health
- Auto-restarting on failure (if configured)

## Stopping a Tunnel

```bash
tunk stop web
# ✓ Tunnel stopped
```

Graceful shutdown with timeout (3 seconds), then force-killed.

## Checking Status

```bash
tunk status web
# Tunnel Status
#   Name:    web
#   Running: yes
```

## Listing Tunnels

```bash
tunk list
# Configured Tunnels
#   web     running
#   api     stopped
```

## Reloading Configuration

Reload tunnels and backends from disk without restarting the daemon:

```bash
tunk reload
# ✓ Daemon configuration reloaded
```

Useful when adding/editing tunnel configs.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TUNK_SOCK_PATH` | `/tmp/tunk.sock` | Daemon socket path |

## Signal Handling

Send Ctrl+C to the daemon to gracefully shutdown all tunnels:

```bash
# Find daemon PID
pgrep tunkd

# Send Ctrl+C equivalent
kill -INT $(pgrep tunkd)
```

The daemon will:
1. Stop accepting new connections
2. Signal all tunnel managers to shutdown
3. Wait up to 5 seconds for graceful shutdown per tunnel
4. Force-kill any remaining tunnels
