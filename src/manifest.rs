use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Package manifest fetched from R2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub repository: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub commit: String,
    pub built_at: String,
    pub system: String,
    pub outputs: HashMap<String, Output>,
    #[serde(default)]
    pub attestation: Option<Attestation>,
}

/// A single output (e.g., "default", "cli", "gui")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub store_path: String,
    pub nar_hash: String,
    pub nar_size: u64,
    #[serde(default)]
    pub bin: Option<String>,
    pub closure: Vec<ClosureEntry>,
}

/// A dependency in the closure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureEntry {
    pub store_path: String,
    pub nar_hash: String,
    pub nar_size: u64,
}

/// GitHub OIDC attestation (optional for MVP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub github_oidc: Option<String>,
    pub github_repository: Option<String>,
    pub github_sha: Option<String>,
    pub github_ref: Option<String>,
}

impl Manifest {
    /// Get an output by name, or error if not found
    pub fn get_output(&self, name: &str) -> eyre::Result<&Output> {
        self.outputs.get(name).ok_or_else(|| {
            let available: Vec<&str> = self.outputs.keys().map(|s| s.as_str()).collect();
            eyre::eyre!(
                "output '{name}' not found. Available outputs: {}",
                available.join(", ")
            )
        })
    }
}

/// Detect the current system in Nix format
pub fn current_system() -> eyre::Result<&'static str> {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    return Ok("x86_64-linux");

    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    return Ok("aarch64-linux");

    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    return Ok("x86_64-darwin");

    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    return Ok("aarch64-darwin");

    #[cfg(not(any(
        all(target_arch = "x86_64", target_os = "linux"),
        all(target_arch = "aarch64", target_os = "linux"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "aarch64", target_os = "macos"),
    )))]
    eyre::bail!("unsupported system: {}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_manifest() {
        let json = r#"{
            "schema_version": 1,
            "repository": "github:owner/repo",
            "ref": "main",
            "commit": "abc123",
            "built_at": "2024-01-15T10:30:00Z",
            "system": "x86_64-linux",
            "outputs": {
                "default": {
                    "store_path": "/home/x/.x/store/abc123-foo",
                    "nar_hash": "sha256-XXXX",
                    "nar_size": 1234,
                    "bin": "bin/foo",
                    "closure": []
                }
            }
        }"#;

        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.git_ref, "main");
        assert!(manifest.outputs.contains_key("default"));
    }

    #[test]
    fn test_get_output_missing() {
        let manifest = Manifest {
            schema_version: 1,
            repository: "test".to_string(),
            git_ref: "main".to_string(),
            commit: "abc".to_string(),
            built_at: "2024-01-01".to_string(),
            system: "x86_64-linux".to_string(),
            outputs: HashMap::new(),
            attestation: None,
        };

        assert!(manifest.get_output("nonexistent").is_err());
    }
}
