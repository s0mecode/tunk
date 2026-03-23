#!/usr/bin/env bash

cd /tmp

git clone https://github.com/s0mecode/tunk

cd tunk

set -e

echo "Building release..."
cargo build --release

echo "Installing binaries..."
mkdir -p ~/.local/bin
cp ./target/release/tunk ~/.local/bin/
cp ./target/release/tunkd ~/.local/bin/

echo "Setting up systemd service..."
mkdir -p ~/.config/systemd/user/

cat << 'EOF' > ~/.config/systemd/user/tunkd.service
[Unit]
Description=Docker-like Tunnel Manager's Daemon
After=network.target

[Service]
Type=simple
ExecStart=%h/.local/bin/tunkd
Restart=on-failure
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
EOF

echo "Reloading daemon and enabling service..."
systemctl --user daemon-reload
systemctl --user enable --now tunkd.service

echo "Done."