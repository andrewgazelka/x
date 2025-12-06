//! Store path management
//!
//! Packages are stored at a fixed path per OS to ensure Nix store paths match
//! between build time and runtime:
//! - Linux: `/home/.x/store` (top-level in /home, avoids per-user paths)
//! - macOS: `/Users/Shared/.x/store` (writable by all users without root)
//!
//! IMPORTANT: These paths are BAKED INTO packages at build time by the GitHub Action.
//! Changing them requires rebuilding all packages.

use std::path::PathBuf;

/// Store path for Linux systems - top-level /home/.x avoids per-user paths
#[cfg(target_os = "linux")]
pub const STORE_PATH: &str = "/home/.x/store";

/// Store path for macOS - /Users/Shared is writable by all users without root
#[cfg(target_os = "macos")]
pub const STORE_PATH: &str = "/Users/Shared/.x/store";

/// Fallback for other systems (Windows, etc.)
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub const STORE_PATH: &str = "/tmp/.x/store";

/// Get the store directory path
pub fn store_dir() -> PathBuf {
    PathBuf::from(STORE_PATH)
}

/// Check if a store path exists
pub fn path_exists(store_path: &str) -> bool {
    std::path::Path::new(store_path).exists()
}

/// Extract the hash from a store path.
///
/// Store paths look like: `/home/x/.x/store/abc123def456-package-name`
/// This returns: `abc123def456`
pub fn hash_from_store_path(store_path: &str) -> eyre::Result<&str> {
    let name = store_path
        .rsplit('/')
        .next()
        .ok_or_else(|| eyre::eyre!("invalid store path: '{store_path}'"))?;

    name.split('-')
        .next()
        .ok_or_else(|| eyre::eyre!("invalid store path format: '{store_path}'"))
}

/// Get the base directory for x data (parent of store)
#[cfg(target_os = "linux")]
pub fn base_dir() -> PathBuf {
    PathBuf::from("/home/.x")
}

#[cfg(target_os = "macos")]
pub fn base_dir() -> PathBuf {
    PathBuf::from("/Users/Shared/.x")
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn base_dir() -> PathBuf {
    PathBuf::from("/tmp/.x")
}

/// Get the directory for cached manifests
pub fn meta_dir() -> PathBuf {
    base_dir().join("meta")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_from_store_path_linux_style() {
        let hash = hash_from_store_path("/home/.x/store/abc123-package-1.0.0").unwrap();
        assert_eq!(hash, "abc123");
    }

    #[test]
    fn test_hash_from_store_path_macos_style() {
        let hash = hash_from_store_path("/Users/Shared/.x/store/abc123-package-1.0.0").unwrap();
        assert_eq!(hash, "abc123");
    }

    #[test]
    fn test_hash_from_complex_path() {
        let hash = hash_from_store_path("/home/.x/store/xyz789def-my-app-name-2.3.4").unwrap();
        assert_eq!(hash, "xyz789def");
    }

    #[test]
    fn test_store_dir() {
        assert_eq!(store_dir().to_str().unwrap(), STORE_PATH);
    }
}
