<p align="center">
  <img src=".github/assets/header.svg" alt="x" width="100%"/>
</p>

<p align="center">
  <code>x run github:owner/repo</code>
</p>

Run Nix packages without installing Nix.

## Features

- **No Nix required**: Run pre-built Nix packages without the Nix package manager
- **Cross-platform**: Linux and macOS (x86_64 and ARM64)
- **Rootless on macOS**: No sudo needed
- **Fast**: Pre-built binaries from CDN with hash verification

## Usage

```bash
x run github:owner/repo           # run default output
x run github:owner/repo#cli       # run specific output
x run github:owner/repo/v1.0.0    # run specific version
x run github:owner/repo -- --help # pass arguments

x info github:owner/repo          # show package info
x list                            # list installed packages
```

## Setup

**macOS** - no root needed:
```bash
mkdir -p /Users/Shared/.x/store
```

**Linux** - one-time sudo:
```bash
sudo mkdir -p /home/x && sudo chown $USER /home/x
mkdir -p /home/x/.x/store
```

## Publishing Packages

Add to `.github/workflows/x-publish.yml`:

```yaml
name: Publish to x
on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  publish:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
          - os: ubuntu-24.04-arm
          - os: macos-13
          - os: macos-14
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: andrewgazelka/x/action@main
        with:
          r2_account_id: ${{ secrets.R2_ACCOUNT_ID }}
          r2_access_key: ${{ secrets.R2_ACCESS_KEY }}
          r2_secret_key: ${{ secrets.R2_SECRET_KEY }}
```

### Required Secrets

| Secret | Description |
|--------|-------------|
| `R2_ACCOUNT_ID` | Cloudflare account ID |
| `R2_ACCESS_KEY` | R2 access key ID |
| `R2_SECRET_KEY` | R2 secret access key |

### Action Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `r2_account_id` | Yes | - | Cloudflare R2 account ID |
| `r2_access_key` | Yes | - | R2 access key ID |
| `r2_secret_key` | Yes | - | R2 secret access key |
| `r2_bucket` | No | `x-pkg` | R2 bucket name |
| `outputs` | No | all | Comma-separated outputs to build |
| `working_directory` | No | `.` | Directory with flake.nix |

## How It Works

```
GitHub Action (x-publish)     R2 CDN                    x CLI
─────────────────────────     ──────                    ─────
Build with Nix           ──▶  /nar/{hash}.nar.xz   ◀── Fetch NARs
Upload NARs + manifest   ──▶  /meta/github/...     ◀── Read manifest
                                                       Unpack to store
                                                       Execute binary
```

Store paths:
- **Linux**: `/home/x/.x/store`
- **macOS**: `/Users/Shared/.x/store`
