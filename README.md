<p align="center">
  <img src=".github/assets/header.svg" alt="x" width="100%"/>
</p>

<p align="center">
  <code>x run github:owner/repo</code>
</p>

Run Nix packages without installing Nix. No root required.

## Features

- **Zero setup**: Downloads pre-built binaries from R2, unpacks NAR archives, runs them
- **Cross-platform**: Works on Linux (x86_64, aarch64) and macOS (Intel, Apple Silicon)
- **Nix-compatible**: Uses the same store path semantics as Nix for binary cache compatibility

## How It Works

Publishers build packages with a GitHub Action that uploads NAR archives to R2. The `x` CLI fetches manifests and dependencies on demand, unpacking them to a fixed store path (`/home/x/.x/store`).

## Usage

```bash
# Run a package
x run github:owner/repo

# Run specific output
x run github:owner/repo#cli

# Run specific version
x run github:owner/repo/v1.0.0

# Pass arguments
x run github:owner/repo -- --help

# Show package info
x info github:owner/repo
```

## Status

MVP in development. Core CLI functionality works:
- Package reference parsing
- Manifest fetching
- NAR unpacking
- Binary execution

Not yet implemented:
- GitHub Action for publishing
- R2 bucket setup
- Garbage collection
- Lockfiles
