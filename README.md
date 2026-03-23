# tunk - Docker-like Tunnel Manager

> Simple, declarative, scalable tunnel management

**tunk** is a daemon-based tunnel manager with a client-daemon architecture. It manages SSH tunnels through a plugin-based backend system, allowing you to define tunnels declaratively and control them via CLI.

## Quick Start

```bash
# Install (requires systemd)
curl -fsSL https://cdn.nilla.run/i/tunk-sd | bash

# Create a backend config ( ~/.config/tunk/backends/my-ssh.ron )
SecureShell(
  (ip: "1.2.3.4", port: 22),
  (username: "user", key_path: Some("/home/user/.ssh/id_rsa"), password: None),
)

# Create a tunnel config ( ~/.config/tunk/tunnels/web.ron )
(
  backend: File("my-ssh"),
  direction: Reverse,
  local: (ip: "127.0.0.1", port: 8080),
  remote: (ip: "0.0.0.0", port: 80),
)

# Manage tunnels
tunk list          # List all tunnels
tunk start web     # Start tunnel
tunk stop web      # Stop tunnel
tunk restart web   # Restart tunnel
tunk status web    # Check status
tunk reload        # Reload configs
```

## Architecture

- **tunk** (CLI): Client sending commands to daemon via Unix socket
- **tunkd** (Daemon): Manages tunnel lifecycle, handles auto-restart
- **tunk-backends**: Pluggable tunnel providers (SSH supported)

## Documentation

- [Installation](docs/installation.md)
- [Configuration](docs/configuration.md)
- [Usage](docs/usage.md)
- [Architecture](docs/architecture.md)
