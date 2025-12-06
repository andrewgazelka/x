# x - Rootless Nix Package Runner

## CRITICAL: Custom Store Path is REQUIRED

**x MUST be 100% rootless. No sudo. No exceptions.**

The GitHub Action builder MUST build packages with a custom store path (e.g., `/Users/Shared/.x/store`), NOT `/nix/store`.

### Why This is Non-Negotiable

Nix packages have hardcoded store paths baked into:
- Binary executables (compiled paths)
- Wrapper scripts (shebang lines, PATH references)
- Dynamic library references (RPATH, ld paths)
- Configuration files

If we build with `/nix/store` paths, users would need `sudo` to create `/nix`. **This defeats the entire purpose of x.**

### The Solution: Build with Custom Store Path

The GitHub Action builder MUST use:

```bash
nix build --store 'local?store=/Users/Shared/.x/store&state=/Users/Shared/.x/state&log=/Users/Shared/.x/log' .#package
```

**This requires rebuilding ALL packages from source** - you CANNOT use prebuilt binaries from `cache.nixos.org` because they have `/nix/store` baked in.

**This is perfectly OK and expected:**
- We have our own R2 binary cache
- Packages only need to be built once per version
- Build time is a one-time cost, not a user cost

### What Does NOT Work

1. **Symlinks from /nix/store to custom path**: Packages have paths baked in at compile time
2. **Runtime path rewriting**: Would need to patch every binary, script, and library - not feasible
3. **nix-portable on macOS**: Linux-only (uses user namespaces, proot, bubblewrap)
4. **Any approach requiring sudo**: Violates the core requirement

### Store Path Convention

These paths are BAKED INTO packages at build time. Changing them requires rebuilding all packages.

- macOS: `/Users/Shared/.x/store` (writable by all users without root)
- Linux: `/opt/.x/store` (requires one-time `sudo mkdir -p /opt/.x && sudo chown $USER /opt/.x`)

## Code Structure

- `crates/libx/core/src/fetch.rs` - Fetches manifests and NARs from R2, unpacks to store
- `crates/libx/core/src/exec.rs` - Executes binaries from the store
- `crates/libx/nar/src/lib.rs` - NAR archive unpacking
- `crates/libx/store/src/lib.rs` - Store path utilities
- `action/action.yml` - GitHub Action for building and publishing packages
