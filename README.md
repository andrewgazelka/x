<p align="center">
  <img src=".github/assets/header.svg" alt="x" width="100%"/>
</p>

<p align="center">
  <code>x run github:owner/repo</code>
</p>

Run Nix packages without root access or installing Nix.

## Features

- **Rootless**: No sudo for daily use after one-time setup
- **Cross-platform**: Works on Linux and macOS
- **Self-contained**: Single binary, no Nix installation required
- **Fast**: Fetches pre-built packages from CDN with hash verification

## Usage

```bash
# Run a package
x run github:owner/repo

# Run with arguments
x run github:owner/repo -- --help

# Run a specific output
x run github:owner/repo#cli

# Run a specific ref
x run github:owner/repo/v1.0.0

# Show package info
x info github:owner/repo

# List installed packages
x list
```

## How It Works

Pre-built packages are fetched from a CDN as NAR archives (Nix's archive format), unpacked to `/home/x/.x/store`, and executed directly. The fixed store path ensures binaries work without patching.

## Status

MVP. Works for basic use cases. Not yet published.
