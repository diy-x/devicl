#!/bin/bash

# Devicl File Cleanup Manager - Installation Script
# This script installs devicl as a systemd service

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SERVICE_NAME="devicl"
INSTALL_DIR="/opt/devicl"
BINARY_NAME="devicl"

# Print colored messages
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    print_error "Please run as root (use sudo)"
    exit 1
fi

print_info "Starting Devicl installation..."

# Look for binary in current directory
if [ -f "./$BINARY_NAME" ]; then
    BINARY_PATH="./$BINARY_NAME"
    print_info "Found binary in current directory"
else
    print_error "Binary '$BINARY_NAME' not found in current directory"
    print_info "Please ensure you have extracted the release package correctly"
    print_info "The directory should contain:"
    print_info "  - devicl (binary)"
    print_info "  - install.sh (this script)"
    print_info "  - uninstall.sh"
    exit 1
fi

# Verify binary is executable
if [ ! -x "$BINARY_PATH" ]; then
    print_info "Making binary executable..."
    chmod +x "$BINARY_PATH"
fi

# Create installation directory
print_info "Creating installation directory: $INSTALL_DIR"
mkdir -p $INSTALL_DIR

# Copy binary
print_info "Installing binary to $INSTALL_DIR/$BINARY_NAME"
cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Create systemd service file
print_info "Creating systemd service"
cat > /etc/systemd/system/${SERVICE_NAME}.service << 'EOF'
[Unit]
Description=Devicl File Cleanup Manager
Documentation=https://github.com/diy-x/devicl
After=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/devicl
Environment="RUST_LOG=info"
ExecStart=/opt/devicl/devicl
Restart=on-failure
RestartSec=5s

# Security settings
PrivateTmp=true

# Resource limits
LimitNOFILE=65536
TasksMax=4096

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
print_info "Reloading systemd daemon"
systemctl daemon-reload

# Ask if user wants to enable and start the service
read -p "Do you want to enable and start the service now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_info "Enabling service to start on boot"
    systemctl enable $SERVICE_NAME

    print_info "Starting service"
    systemctl start $SERVICE_NAME

    # Wait a moment for the service to start
    sleep 2

    # Check service status
    if systemctl is-active --quiet $SERVICE_NAME; then
        print_info "Service is running!"
        print_info "Service status:"
        systemctl status $SERVICE_NAME --no-pager
    else
        print_error "Service failed to start. Check logs with: journalctl -u $SERVICE_NAME -n 50"
        exit 1
    fi
else
    print_info "Service installed but not started"
    print_info "To enable: sudo systemctl enable $SERVICE_NAME"
    print_info "To start: sudo systemctl start $SERVICE_NAME"
fi

print_info ""
print_info "Installation complete!"
print_info ""
print_info "Useful commands:"
print_info "  Start service:   sudo systemctl start $SERVICE_NAME"
print_info "  Stop service:    sudo systemctl stop $SERVICE_NAME"
print_info "  Restart service: sudo systemctl restart $SERVICE_NAME"
print_info "  Service status:  sudo systemctl status $SERVICE_NAME"
print_info "  View logs:       sudo journalctl -u $SERVICE_NAME -f"
print_info "  Enable on boot:  sudo systemctl enable $SERVICE_NAME"
print_info "  Disable on boot: sudo systemctl disable $SERVICE_NAME"
print_info ""
print_info "Application will be available at: http://localhost:3000"
print_info "Default password: holomotion"
print_info ""
print_warn "IMPORTANT: Please change the default password after first login!"
print_info ""
