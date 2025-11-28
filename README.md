# Bluetooth Widget

A lightweight GTK4-based Bluetooth device manager for Linux.

## What It Does

Displays and manages Bluetooth devices with a simple, clean interface.

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Architecture

- `main.rs` - Entry point and application lifecycle
- `ui/` - GTK4 interface components
- `bluetooth.rs` - Bluetooth device communication
- `config.rs` - Configuration management

