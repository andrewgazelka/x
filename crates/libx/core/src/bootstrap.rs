//! First-time setup and bootstrapping
//!
//! Sets up the store directory:
//! - Linux: `/home/x/.x/store` (requires one-time sudo)
//! - macOS: `/Users/Shared/.x/store` (no root needed!)

use eyre::WrapErr;

/// Check if the store is set up, and set it up if not
pub fn ensure_store() -> eyre::Result<()> {
    let store_dir = libx_store::store_dir();

    if store_dir.exists() {
        return Ok(());
    }

    tracing::info!(
        "First-time setup: creating store at {}",
        store_dir.display()
    );

    #[cfg(target_os = "linux")]
    setup_linux()?;

    #[cfg(target_os = "macos")]
    setup_macos()?;

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    eyre::bail!("unsupported operating system");

    Ok(())
}

#[cfg(target_os = "linux")]
fn setup_linux() -> eyre::Result<()> {
    let home_x = std::path::Path::new("/home/x");
    let store_dir = libx_store::store_dir();

    if !home_x.exists() {
        tracing::info!("Creating /home/x (requires sudo once)...");

        let status = std::process::Command::new("sudo")
            .args(["mkdir", "-p", "/home/x"])
            .status()
            .wrap_err("failed to run 'sudo mkdir -p /home/x'")?;

        if !status.success() {
            eyre::bail!(
                "failed to create /home/x. Please run:\n\
                 sudo mkdir -p /home/x && sudo chown $USER /home/x"
            );
        }

        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let status = std::process::Command::new("sudo")
            .args(["chown", &user, "/home/x"])
            .status()
            .wrap_err_with(|| format!("failed to run 'sudo chown {user} /home/x'"))?;

        if !status.success() {
            eyre::bail!("failed to chown /home/x to {user}");
        }
    }

    std::fs::create_dir_all(&store_dir)
        .wrap_err_with(|| format!("failed to create store directory {}", store_dir.display()))?;

    tracing::info!("Store created at {}", store_dir.display());
    Ok(())
}

#[cfg(target_os = "macos")]
fn setup_macos() -> eyre::Result<()> {
    // /Users/Shared is writable by all users without root!
    let store_dir = libx_store::store_dir();

    std::fs::create_dir_all(&store_dir)
        .wrap_err_with(|| format!("failed to create store directory {}", store_dir.display()))?;

    tracing::info!("Store created at {}", store_dir.display());
    Ok(())
}

/// Print setup instructions for the user
pub fn print_manual_setup_instructions() {
    #[cfg(target_os = "linux")]
    {
        tracing::info!("\n=== Manual Setup Instructions ===\n");
        tracing::info!("On Linux, run:");
        tracing::info!("  sudo mkdir -p /home/x && sudo chown $USER /home/x");
        tracing::info!("  mkdir -p /home/x/.x/store");
    }

    #[cfg(target_os = "macos")]
    {
        tracing::info!("\n=== Manual Setup Instructions ===\n");
        tracing::info!("On macOS, run:");
        tracing::info!("  mkdir -p /Users/Shared/.x/store");
        tracing::info!("\nNo root access required!");
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        tracing::warn!("Manual setup instructions not available for this platform");
    }
}
