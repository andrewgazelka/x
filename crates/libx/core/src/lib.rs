//! Rootless, cross-platform Nix package runner.
//!
//! This crate provides the core functionality for fetching and running
//! Nix packages without requiring root access or a full Nix installation.

pub mod bootstrap;
pub mod exec;
pub mod fetch;

// Re-export from sub-crates for convenience
pub use libx_manifest as manifest;
pub use libx_nar as nar;
pub use libx_ref as ref_parser;
pub use libx_store as store;

use eyre::WrapErr;

/// Run a package with the given arguments.
pub fn run(package: &str, args: &[String]) -> eyre::Result<()> {
    bootstrap::ensure_store().wrap_err("failed to ensure store exists")?;

    let pkg_ref = libx_ref::parse(package)
        .wrap_err_with(|| format!("failed to parse package reference '{package}'"))?;
    let manifest = fetch::fetch_manifest(&pkg_ref)
        .wrap_err_with(|| format!("failed to fetch manifest for '{package}'"))?;
    let output = manifest
        .get_output(&pkg_ref.output)
        .wrap_err_with(|| format!("failed to get output '{}' from manifest", pkg_ref.output))?;

    fetch::ensure_closure(output)
        .wrap_err_with(|| format!("failed to fetch closure for '{package}'"))?;
    exec::run(output, args)
}

/// Get information about a package.
pub fn info(package: &str) -> eyre::Result<PackageInfo> {
    let pkg_ref = libx_ref::parse(package)
        .wrap_err_with(|| format!("failed to parse package reference '{package}'"))?;
    let manifest = fetch::fetch_manifest(&pkg_ref)
        .wrap_err_with(|| format!("failed to fetch manifest for '{package}'"))?;

    let outputs = manifest
        .outputs
        .iter()
        .map(|(name, output)| OutputInfo {
            name: name.clone(),
            store_path: output.store_path.clone(),
            closure_size: output.closure.len(),
            bin: output.bin.clone(),
        })
        .collect();

    Ok(PackageInfo {
        repository: manifest.repository,
        git_ref: manifest.git_ref,
        commit: manifest.commit,
        built_at: manifest.built_at,
        system: manifest.system,
        outputs,
    })
}

/// List installed store paths.
pub fn list() -> eyre::Result<Vec<String>> {
    let store_dir = libx_store::store_dir();

    if !store_dir.exists() {
        return Ok(vec![]);
    }

    let entries: Vec<String> = std::fs::read_dir(&store_dir)
        .wrap_err_with(|| format!("failed to read store directory {}", store_dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    Ok(entries)
}

/// Package information returned by [`info`].
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub repository: String,
    pub git_ref: String,
    pub commit: String,
    pub built_at: String,
    pub system: String,
    pub outputs: Vec<OutputInfo>,
}

/// Information about a single package output.
#[derive(Debug, Clone)]
pub struct OutputInfo {
    pub name: String,
    pub store_path: String,
    pub closure_size: usize,
    pub bin: Option<String>,
}
