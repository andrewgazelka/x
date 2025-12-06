//! Binary execution
//!
//! Packages have custom store paths baked in (e.g., /Users/Shared/.x/store).
//! The store_path in the manifest is the actual path on disk.

use eyre::WrapErr;
use std::path::Path;

/// Run the binary from an output, replacing the current process
pub fn run(output: &libx_manifest::Output, args: &[String]) -> eyre::Result<()> {
    let bin_path = resolve_binary(output)?;

    tracing::info!("Running {bin_path}...");

    // exec() replaces the current process on Unix
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let err = std::process::Command::new(&bin_path).args(args).exec();

        // If we get here, exec failed
        eyre::bail!("failed to exec '{bin_path}': {err}");
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, spawn and wait
        let status = std::process::Command::new(&bin_path)
            .args(args)
            .status()
            .wrap_err_with(|| format!("failed to spawn '{bin_path}'"))?;

        if !status.success() {
            // Return error instead of exit to allow graceful handling
            eyre::bail!("process exited with status: {}", status.code().unwrap_or(1));
        }

        Ok(())
    }
}

/// Resolve the path to the binary for an output.
/// The store_path already points to the correct location on disk.
fn resolve_binary(output: &libx_manifest::Output) -> eyre::Result<String> {
    let store_path = Path::new(&output.store_path);

    // If bin is specified, use it
    if let Some(bin) = &output.bin {
        return Ok(format!("{}/{bin}", store_path.display()));
    }

    // Otherwise, try to find a single binary in bin/
    let bin_dir = store_path.join("bin");
    find_single_binary(&bin_dir.to_string_lossy())
}

/// Find a single binary in a directory
fn find_single_binary(bin_dir: &str) -> eyre::Result<String> {
    let path = std::path::Path::new(bin_dir);

    if !path.exists() {
        eyre::bail!("bin directory not found: {bin_dir}");
    }

    let entries: Vec<_> = std::fs::read_dir(path)
        .wrap_err_with(|| format!("failed to read directory {bin_dir}"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file() || t.is_symlink()))
        .collect();

    match entries.len() {
        0 => eyre::bail!("no binaries found in {bin_dir}"),
        1 => {
            let Some(entry) = entries.first() else {
                unreachable!("checked len is 1")
            };
            Ok(entry.path().to_string_lossy().to_string())
        }
        n => {
            let names: Vec<_> = entries
                .iter()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            eyre::bail!(
                "multiple binaries found ({n}) in {bin_dir}: {}\n\
                 Specify which one with #output",
                names.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_binary_with_explicit_bin() {
        // Store paths now contain the actual path (no rewriting needed)
        let store_path = format!("{}/abc123-foo", libx_store::STORE_PATH);
        let output = libx_manifest::Output {
            store_path: store_path.clone(),
            nar_hash: "sha256-xxx".to_string(),
            nar_size: 100,
            bin: Some("bin/my-binary".to_string()),
            closure: vec![],
        };

        let path = resolve_binary(&output).unwrap();
        let expected = format!("{}/bin/my-binary", store_path);
        assert_eq!(path, expected);
    }
}
