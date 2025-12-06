//! HTTP fetching for manifests and NARs

use eyre::WrapErr;

/// Base URL for the x package registry
pub const R2_BASE: &str = "https://pub-698579286ab3445b8062024bd63d5bf3.r2.dev";

/// Fetch a manifest for a package reference
pub fn fetch_manifest(pkg: &libx_ref::Package) -> eyre::Result<libx_manifest::Manifest> {
    let system = libx_manifest::current_system()?;
    let url = format!(
        "{R2_BASE}/meta/{}/{}/{}/{}-{system}.json",
        pkg.host, pkg.owner, pkg.repo, pkg.git_ref
    );

    tracing::info!("Fetching manifest from {url}...");

    let resp = reqwest::blocking::get(&url)
        .wrap_err_with(|| format!("failed to fetch manifest from {url}"))?;

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

    let manifest: libx_manifest::Manifest = resp
        .json()
        .wrap_err_with(|| format!("failed to parse manifest JSON from {url}"))?;
    Ok(manifest)
}

/// Ensure all paths in the closure are present in the store
pub fn ensure_closure(output: &libx_manifest::Output) -> eyre::Result<()> {
    // First fetch all dependencies
    for dep in &output.closure {
        if !libx_store::path_exists(&dep.store_path) {
            fetch_and_unpack(&dep.store_path, &dep.nar_hash)
                .wrap_err_with(|| format!("failed to fetch dependency {}", dep.store_path))?;
        }
    }

    // Then fetch the output itself
    if !libx_store::path_exists(&output.store_path) {
        fetch_and_unpack(&output.store_path, &output.nar_hash)
            .wrap_err_with(|| format!("failed to fetch output {}", output.store_path))?;
    }

    Ok(())
}

/// Fetch a NAR from R2 and unpack it to the store
fn fetch_and_unpack(store_path: &str, expected_hash: &str) -> eyre::Result<()> {
    let hash = libx_store::hash_from_store_path(store_path)
        .wrap_err_with(|| format!("failed to extract hash from store path '{store_path}'"))?;
    let url = format!("{R2_BASE}/nar/{hash}.nar.xz");

    tracing::info!("Fetching {store_path}...");

    let resp =
        reqwest::blocking::get(&url).wrap_err_with(|| format!("failed to fetch NAR from {url}"))?;

    if !resp.status().is_success() {
        eyre::bail!("failed to fetch NAR {hash}: HTTP {}", resp.status());
    }

    let compressed = resp
        .bytes()
        .wrap_err_with(|| format!("failed to read response body from {url}"))?;

    // Decompress XZ
    let mut decoder = liblzma::read::XzDecoder::new(compressed.as_ref());
    let mut nar_data = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut nar_data)
        .wrap_err_with(|| format!("failed to decompress XZ data for {store_path}"))?;

    // Verify hash
    verify_hash(&nar_data, expected_hash)
        .wrap_err_with(|| format!("hash verification failed for {store_path}"))?;

    // Create destination directory
    let dest = std::path::PathBuf::from(store_path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .wrap_err_with(|| format!("failed to create parent directory for {store_path}"))?;
    }

    // Unpack NAR
    libx_nar::unpack(nar_data.as_slice(), &dest)
        .wrap_err_with(|| format!("failed to unpack NAR to {store_path}"))?;

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
        let expected = "sha256-uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=";
        assert!(verify_hash(data, expected).is_ok());
    }

    #[test]
    fn test_verify_hash_incorrect() {
        let data = b"hello world";
        let expected = "sha256-wronghash";
        assert!(verify_hash(data, expected).is_err());
    }
}
