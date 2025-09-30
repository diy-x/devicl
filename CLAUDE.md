# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`devicl` is a Rust project currently in early development stage with minimal code structure.

## Common Commands

### Build
```bash
cargo build
```

### Run
```bash
cargo run
```

### Test
```bash
cargo test
```

### Check (fast compile check without building)
```bash
cargo check
```

### Development with auto-rebuild
```bash
cargo watch -x run  # requires cargo-watch
```

## Code Architecture

This is a new Rust project with a standard Cargo structure:
- `src/main.rs` - Entry point for the binary application
- `Cargo.toml` - Project manifest and dependencies (currently no dependencies)
- Edition 2024 is specified (note: this is a future edition; verify this is intentional or use 2021)