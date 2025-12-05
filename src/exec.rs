//! Binary execution

use crate::manifest;

/// Run the binary from an output, replacing the current process
pub fn run(output: &manifest::Output, args: &[String]) -> eyre::Result<()> {
    let bin_path = resolve_binary(output)?;

    eprintln!("Running {bin_path}...");

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
        let status = std::process::Command::new(&bin_path).args(args).status()?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}

/// Resolve the path to the binary for an output
fn resolve_binary(output: &manifest::Output) -> eyre::Result<String> {
    // If bin is specified, use it
    if let Some(bin) = &output.bin {
        return Ok(format!("{}/{}", output.store_path, bin));
    }

    // Otherwise, try to find a single binary in bin/
    let bin_dir = format!("{}/bin", output.store_path);
    find_single_binary(&bin_dir)
}

/// Find a single binary in a directory
fn find_single_binary(bin_dir: &str) -> eyre::Result<String> {
    let path = std::path::Path::new(bin_dir);

    if !path.exists() {
        eyre::bail!("bin directory not found: {bin_dir}");
    }

    let entries: Vec<_> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file() || t.is_symlink()))
        .collect();

    match entries.len() {
        0 => eyre::bail!("no binaries found in {bin_dir}"),
        1 => Ok(entries[0].path().to_string_lossy().to_string()),
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
        let output = manifest::Output {
            store_path: "/home/x/.x/store/abc123-foo".to_string(),
            nar_hash: "sha256-xxx".to_string(),
            nar_size: 100,
            bin: Some("bin/my-binary".to_string()),
            closure: vec![],
        };

        let path = resolve_binary(&output).unwrap();
        assert_eq!(path, "/home/x/.x/store/abc123-foo/bin/my-binary");
    }
}
