#!/bin/bash

# Devicl Build and Package Script
# This script builds the project and creates a release package

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Configuration
BINARY_NAME="devicl"
RELEASE_DIR="release"

# Detect architecture
ARCH=$(uname -m)
print_info "Detected architecture: $ARCH"

# Ask user which architecture to build for
echo ""
echo "Select build target:"
echo "1) x86_64 (64-bit Intel/AMD)"
echo "2) aarch64 (ARM64)"
echo "3) Both"
read -p "Enter choice [1-3]: " choice

case $choice in
    1)
        TARGETS=("x86_64")
        ;;
    2)
        TARGETS=("aarch64")
        ;;
    3)
        TARGETS=("x86_64" "aarch64")
        ;;
    *)
        print_error "Invalid choice"
        exit 1
        ;;
esac

# Build function
build_target() {
    local target=$1
    local target_arg=""
    local target_dir=""

    if [ "$target" = "x86_64" ]; then
        target_arg=""
        target_dir="target/release"
        print_step "Building for x86_64..."
    else
        target_arg="--target aarch64-unknown-linux-gnu"
        target_dir="target/aarch64-unknown-linux-gnu/release"
        print_step "Building for aarch64..."

        # Check if target is installed
        if ! rustup target list | grep -q "aarch64-unknown-linux-gnu (installed)"; then
            print_warn "aarch64 target not installed. Installing..."
            rustup target add aarch64-unknown-linux-gnu
        fi
    fi

    # Build
    print_info "Compiling..."
    cargo build --release $target_arg

    # Create release directory
    local release_subdir="${RELEASE_DIR}/${BINARY_NAME}-${target}-linux"
    print_info "Creating release package in $release_subdir"

    rm -rf "$release_subdir"
    mkdir -p "$release_subdir"

    # Copy binary
    print_info "Copying binary..."
    cp "${target_dir}/${BINARY_NAME}" "$release_subdir/"

    # Strip binary to reduce size
    print_info "Stripping binary to reduce size..."
    if [ "$target" = "x86_64" ]; then
        strip "$release_subdir/${BINARY_NAME}" 2>/dev/null || print_warn "Could not strip binary"
    else
        aarch64-linux-gnu-strip "$release_subdir/${BINARY_NAME}" 2>/dev/null || print_warn "Could not strip binary (aarch64-linux-gnu-strip not found)"
    fi

    # Copy installation scripts
    print_info "Copying installation scripts..."
    cp install.sh "$release_subdir/"
    cp uninstall.sh "$release_subdir/"
    chmod +x "$release_subdir/install.sh"
    chmod +x "$release_subdir/uninstall.sh"

    # Copy documentation
    print_info "Copying documentation..."
    cp INSTALL.md "$release_subdir/" 2>/dev/null || print_warn "INSTALL.md not found"
    cp README.md "$release_subdir/" 2>/dev/null || print_warn "README.md not found"

    # Create archive
    print_info "Creating archive..."
    cd "$RELEASE_DIR"
    tar -czf "${BINARY_NAME}-${target}-linux.tar.gz" "${BINARY_NAME}-${target}-linux"
    cd ..

    # Calculate sizes
    local binary_size=$(du -h "$release_subdir/${BINARY_NAME}" | cut -f1)
    local archive_size=$(du -h "${RELEASE_DIR}/${BINARY_NAME}-${target}-linux.tar.gz" | cut -f1)

    print_info "Binary size: $binary_size"
    print_info "Archive size: $archive_size"
    print_info "Package created: ${RELEASE_DIR}/${BINARY_NAME}-${target}-linux.tar.gz"
    echo ""
}

# Main build process
print_info "Starting build process..."
echo ""

# Create release directory
mkdir -p "$RELEASE_DIR"

# Build each target
for target in "${TARGETS[@]}"; do
    build_target "$target"
done

# Summary
print_info "========================================"
print_info "Build complete!"
print_info "========================================"
echo ""
print_info "Release packages:"
for target in "${TARGETS[@]}"; do
    echo "  - ${RELEASE_DIR}/${BINARY_NAME}-${target}-linux.tar.gz"
done
echo ""
print_info "To install, extract the archive and run:"
print_info "  tar -xzf ${BINARY_NAME}-<arch>-linux.tar.gz"
print_info "  cd ${BINARY_NAME}-<arch>-linux"
print_info "  sudo ./install.sh"
echo ""
