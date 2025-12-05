//! Store path management
//!
//! All packages are stored at a fixed path `/home/x/.x/store` to ensure
//! Nix store paths match between build time and runtime.

/// The fixed store path that ALL components must use.
/// This path is baked into binaries at build time.
pub const STORE_PATH: &str = "/home/x/.x/store";

/// Get the store directory path
pub fn store_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(STORE_PATH)
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
pub fn base_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("/home/x/.x")
}

/// Get the directory for cached manifests
pub fn meta_dir() -> std::path::PathBuf {
    base_dir().join("meta")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_from_store_path() {
        let hash = hash_from_store_path("/home/x/.x/store/abc123-package-1.0.0").unwrap();
        assert_eq!(hash, "abc123");
    }

    #[test]
    fn test_hash_from_complex_path() {
        let hash = hash_from_store_path("/home/x/.x/store/xyz789def-my-app-name-2.3.4").unwrap();
        assert_eq!(hash, "xyz789def");
    }

    #[test]
    fn test_store_dir() {
        assert_eq!(store_dir().to_str().unwrap(), STORE_PATH);
    }
}
