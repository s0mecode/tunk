# Configuration

tunk uses [RON](https://github.com/ron-rs/ron) (Rusty Object Notation) for configuration files.

## Directory Structure

```
~/.config/tunk/
├── backends/      # Backend configurations
│   └── my-ssh.ron
└── tunnels/       # Tunnel configurations
    └── web.ron
```

## Backend Configuration

Backends define how to connect to a tunnel provider. Currently only SSH is supported.

### SSH Backend

Location: `~/.config/tunk/backends/my-ssh.ron`

```rust
SecureShell(
  (ip: "1.2.3.4", port: 22), // Remote server
  (
    username: "tunneluser", // Remote SSH server username
    key_path: Some("/home/user/.ssh/id_ed25519"), // Path to the SSH key, or None for password-only auth
    password: Some("secret"), // or None for key-only auth
  ),
)
```

| Field | Type | Description |
|-------|------|-------------|
| `ip` | IP address | SSH server IP |
| `port` | i16 | SSH server port |
| `username` | String | SSH username |
| `key_path` | Option<Path> | Path to SSH private key |
| `password` | Option<String> | SSH password (not recommended) |

## Tunnel Configuration

Tunnels define port forwarding rules using a backend.

Location: `~/.config/tunk/tunnels/web.ron`

```rust
(
  backend: File("my-ssh"), // Reference to backend
  direction: Reverse, // or Forward
  local: (ip: "127.0.0.1", port: 8080), // Local bind address
  remote: (ip: "0.0.0.0", port: 80), // Remote bind address
  mgr: Some(( // Optional manager config
    auto_start: Some(true),
    auto_restart: Some(true),
    auto_restart_interval: Some(1000),
  )),
)
```

### Tunnel Direction

- **Reverse** (`-R`): Bind on remote, forward to local. Remote listens, traffic sent to local.
- **Forward** (`-L`): Bind on local, forward to remote. Local listens, traffic sent to remote.

### Manager Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_start` | Option<bool> | None | Start tunnel when daemon starts |
| `auto_restart` | Option<bool> | None | Restart tunnel on failure |
| `auto_restart_interval` | Option<u64> | 1000ms | Delay between restart attempts |

### Inline Backend

Instead of referencing a file, you can inline the backend:

```rust
(
  backend: SecureShell(
    (ip: "1.2.3.4", port: 22),
    (username: "user", key_path: None, password: Some("pass")),
  ),
  direction: Reverse,
  local: (ip: "127.0.0.1", port: 8080),
  remote: (ip: "0.0.0.0", port: 80),
)
```

## Examples

See [`examples/`](https://github.com/s0mecode/tunk/tree/main/examples) and [`tunk-backends/examples`](https://github.com/s0mecode/tunk/tree/main/tunk-backends/examples)  for full examples.
