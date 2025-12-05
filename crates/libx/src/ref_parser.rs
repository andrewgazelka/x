/// Parsed package reference from strings like:
/// - `github:owner/repo`
/// - `github:owner/repo#output`
/// - `github:owner/repo/ref`
/// - `github:owner/repo/ref#output`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRef {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub git_ref: String,
    pub output: String,
}

impl PackageRef {
    pub const DEFAULT_REF: &str = "latest";
    pub const DEFAULT_OUTPUT: &str = "default";
}

/// Parse a package reference string into its components.
///
/// # Examples
/// ```
/// let pkg = parse("github:andrewgazelka/tap").unwrap();
/// assert_eq!(pkg.host, "github");
/// assert_eq!(pkg.owner, "andrewgazelka");
/// assert_eq!(pkg.repo, "tap");
/// assert_eq!(pkg.git_ref, "latest");
/// assert_eq!(pkg.output, "default");
/// ```
pub fn parse(input: &str) -> eyre::Result<PackageRef> {
    // Split on ':' to get host and rest
    let (host, rest) = input
        .split_once(':')
        .ok_or_else(|| eyre::eyre!("invalid package reference: missing ':' in '{input}'"))?;

    if host.is_empty() {
        eyre::bail!("invalid package reference: empty host in '{input}'");
    }

    // Split rest on '#' to separate path from output
    let (path, output) = match rest.split_once('#') {
        Some((p, o)) => (p, o.to_string()),
        None => (rest, PackageRef::DEFAULT_OUTPUT.to_string()),
    };

    if output.is_empty() {
        eyre::bail!("invalid package reference: empty output in '{input}'");
    }

    // Split path on '/' to get owner, repo, and optional ref
    let parts: Vec<&str> = path.split('/').collect();

    let (owner, repo, git_ref) = match parts.as_slice() {
        [owner, repo] => (
            (*owner).to_string(),
            (*repo).to_string(),
            PackageRef::DEFAULT_REF.to_string(),
        ),
        [owner, repo, git_ref] => (
            (*owner).to_string(),
            (*repo).to_string(),
            (*git_ref).to_string(),
        ),
        // Handle refs with slashes like "refs/heads/main"
        [owner, repo, ref_parts @ ..] if !ref_parts.is_empty() => (
            (*owner).to_string(),
            (*repo).to_string(),
            ref_parts.join("/"),
        ),
        _ => eyre::bail!(
            "invalid package reference: expected 'host:owner/repo[/ref]', got '{input}'"
        ),
    };

    if owner.is_empty() || repo.is_empty() {
        eyre::bail!("invalid package reference: empty owner or repo in '{input}'");
    }

    Ok(PackageRef {
        host: host.to_string(),
        owner,
        repo,
        git_ref,
        output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ref() {
        let pkg = parse("github:owner/repo").unwrap();
        assert_eq!(pkg.host, "github");
        assert_eq!(pkg.owner, "owner");
        assert_eq!(pkg.repo, "repo");
        assert_eq!(pkg.git_ref, "latest");
        assert_eq!(pkg.output, "default");
    }

    #[test]
    fn test_with_output() {
        let pkg = parse("github:owner/repo#cli").unwrap();
        assert_eq!(pkg.output, "cli");
        assert_eq!(pkg.git_ref, "latest");
    }

    #[test]
    fn test_with_ref() {
        let pkg = parse("github:owner/repo/v1.0.0").unwrap();
        assert_eq!(pkg.git_ref, "v1.0.0");
        assert_eq!(pkg.output, "default");
    }

    #[test]
    fn test_with_ref_and_output() {
        let pkg = parse("github:owner/repo/main#cli").unwrap();
        assert_eq!(pkg.git_ref, "main");
        assert_eq!(pkg.output, "cli");
    }

    #[test]
    fn test_invalid_no_colon() {
        assert!(parse("github/owner/repo").is_err());
    }

    #[test]
    fn test_invalid_empty_output() {
        assert!(parse("github:owner/repo#").is_err());
    }
}
