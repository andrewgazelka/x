//! HTTP fetching for manifests and NARs

use crate::{manifest, nar, ref_parser, store};

/// Base URL for the x package registry
pub const R2_BASE: &str = "https://x-pkg.r2.dev";

/// Fetch a manifest for a package reference
pub fn fetch_manifest(pkg: &ref_parser::PackageRef) -> eyre::Result<manifest::Manifest> {
    let url = format!(
        "{}/meta/{}/{}/{}/{}.json",
        R2_BASE, pkg.host, pkg.owner, pkg.repo, pkg.git_ref
    );

    eprintln!("Fetching manifest from {url}...");

    let resp = reqwest::blocking::get(&url)?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        eyre::bail!(
            "package not found: {}:{}/{}/{}\n\
             URL: {url}",
            pkg.host,
            pkg.owner,
            pkg.repo,
            pkg.git_ref
        );
    }

    if !resp.status().is_success() {
        eyre::bail!("failed to fetch manifest: HTTP {}", resp.status());
    }

    let manifest: manifest::Manifest = resp.json()?;
    Ok(manifest)
}

/// Ensure all paths in the closure are present in the store
pub fn ensure_closure(output: &manifest::Output) -> eyre::Result<()> {
    // First fetch all dependencies
    for dep in &output.closure {
        if !store::path_exists(&dep.store_path) {
            fetch_and_unpack(&dep.store_path, &dep.nar_hash)?;
        }
    }

    // Then fetch the output itself
    if !store::path_exists(&output.store_path) {
        fetch_and_unpack(&output.store_path, &output.nar_hash)?;
    }

    Ok(())
}

/// Fetch a NAR from R2 and unpack it to the store
fn fetch_and_unpack(store_path: &str, expected_hash: &str) -> eyre::Result<()> {
    let hash = store::hash_from_store_path(store_path)?;
    let url = format!("{}/nar/{}.nar.xz", R2_BASE, hash);

    eprintln!("Fetching {store_path}...");

    let resp = reqwest::blocking::get(&url)?;

    if !resp.status().is_success() {
        eyre::bail!("failed to fetch NAR {hash}: HTTP {}", resp.status());
    }

    let compressed = resp.bytes()?;

    // Decompress XZ
    let mut decoder = liblzma::read::XzDecoder::new(&compressed[..]);
    let mut nar_data = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut nar_data)?;

    // Verify hash
    verify_hash(&nar_data, expected_hash)?;

    // Create destination directory
    let dest = std::path::PathBuf::from(store_path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Unpack NAR
    nar::unpack(&nar_data[..], &dest)?;

    Ok(())
}

/// Verify the SHA256 hash of data matches expected
fn verify_hash(data: &[u8], expected: &str) -> eyre::Result<()> {
    use sha2::Digest;

    let hash = sha2::Sha256::digest(data);
    let actual = format!(
        "sha256-{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hash)
    );

    if actual != expected {
        eyre::bail!("hash mismatch: expected {expected}, got {actual}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_hash_correct() {
        let data = b"hello world";
        // SHA256 of "hello world" in base64
        let expected = "sha256-uU0nuZNNPgilLlLX2n2r+gy4C/FVQIykOlMmkjjjP4E=";
        assert!(verify_hash(data, expected).is_ok());
    }

    #[test]
    fn test_verify_hash_incorrect() {
        let data = b"hello world";
        let expected = "sha256-wronghash";
        assert!(verify_hash(data, expected).is_err());
    }
}
