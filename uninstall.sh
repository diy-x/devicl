#!/bin/bash

# Devicl File Cleanup Manager - Uninstallation Script
# This script removes devicl service and files

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SERVICE_NAME="devicl"
INSTALL_DIR="/opt/devicl"
SERVICE_FILE="devicl.service"

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

print_warn "This will completely remove Devicl from your system"
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "Uninstallation cancelled"
    exit 0
fi

print_info "Starting Devicl uninstallation..."

# Stop the service if running
if systemctl is-active --quiet $SERVICE_NAME; then
    print_info "Stopping service..."
    systemctl stop $SERVICE_NAME
fi

# Disable the service if enabled
if systemctl is-enabled --quiet $SERVICE_NAME 2>/dev/null; then
    print_info "Disabling service..."
    systemctl disable $SERVICE_NAME
fi

# Remove systemd service file
if [ -f "/etc/systemd/system/$SERVICE_FILE" ]; then
    print_info "Removing systemd service file..."
    rm -f /etc/systemd/system/$SERVICE_FILE
    systemctl daemon-reload
fi

# Ask about removing data
read -p "Do you want to remove all application data in $INSTALL_DIR? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "$INSTALL_DIR" ]; then
        print_info "Removing installation directory..."
        rm -rf $INSTALL_DIR
    fi
else
    print_info "Keeping application data in $INSTALL_DIR"
    print_warn "Database and configuration remain at: $INSTALL_DIR/devicl.db"
fi

print_info ""
print_info "Uninstallation complete!"
print_info ""

if [ -d "$INSTALL_DIR" ]; then
    print_info "Note: Application data still exists at $INSTALL_DIR"
    print_info "To remove manually: sudo rm -rf $INSTALL_DIR"
fi
