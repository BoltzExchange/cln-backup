# Backup

A CLN plugin that uploads the latest SCB to remote locations.
Currently, it supports S3-compatible APIs and WebDAV.
The plugin will upload the SCB on startup and when a channel is opened or closed.

## Installation

Compiling the plugin requires Rust to be installed.

By default, the plugin is compiled with S3 and WebDAV support:

```bash
cargo build --release
```

To compile only with S3 support:

```bash
cargo build --release --no-default-features --features s3
```

Or only with WebDAV support:

```bash
cargo build --release --no-default-features --features webdav
```
