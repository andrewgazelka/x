//! First-time setup and bootstrapping
//!
//! Sets up the store directory at `/home/x/.x/store`.
//! This path must be consistent across all systems for Nix store paths to work.

use crate::store;

/// Check if the store is set up, and set it up if not
pub fn ensure_store() -> eyre::Result<()> {
    let store_dir = store::store_dir();

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
    let store_dir = store::store_dir();

    if !home_x.exists() {
        tracing::info!("Creating /home/x (requires sudo)...");

        // Try to create with sudo
        let status = std::process::Command::new("sudo")
            .args(["mkdir", "-p", "/home/x"])
            .status()?;

        if !status.success() {
            eyre::bail!(
                "failed to create /home/x. Please run:\n\
                 sudo mkdir -p /home/x && sudo chown $USER /home/x"
            );
        }

        // Change ownership to current user
        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let status = std::process::Command::new("sudo")
            .args(["chown", &user, "/home/x"])
            .status()?;

        if !status.success() {
            eyre::bail!("failed to chown /home/x to {user}");
        }
    }

    // Create the store directory
    std::fs::create_dir_all(&store_dir)?;

    tracing::info!("Store created at {}", store_dir.display());
    Ok(())
}

#[cfg(target_os = "macos")]
fn setup_macos() -> eyre::Result<()> {
    let shared_x = std::path::Path::new("/Users/Shared/.x");
    let home_x = std::path::Path::new("/home/x");
    let store_dir = store::store_dir();

    // Strategy: Use /Users/Shared/.x as the real location,
    // symlink /home/x -> /Users/Shared/.x

    // Create /Users/Shared/.x (no sudo needed)
    if !shared_x.exists() {
        std::fs::create_dir_all(shared_x)?;
    }

    // Create the symlink at /home/x if it doesn't exist
    if !home_x.exists() {
        tracing::info!("Creating symlink /home/x -> /Users/Shared/.x (requires sudo)...");

        // Ensure /home exists (may need synthetic.conf on newer macOS)
        let home = std::path::Path::new("/home");
        if !home.exists() {
            tracing::warn!(
                "Note: /home doesn't exist on macOS.\n\
                 You may need to add 'home' to /etc/synthetic.conf and reboot.\n\
                 Alternatively, we'll try to create it with sudo."
            );

            let status = std::process::Command::new("sudo")
                .args(["mkdir", "-p", "/home"])
                .status()?;

            if !status.success() {
                eyre::bail!(
                    "failed to create /home. On macOS, you may need to:\n\
                     1. Add 'home' to /etc/synthetic.conf\n\
                     2. Reboot\n\
                     3. Run x again"
                );
            }
        }

        let status = std::process::Command::new("sudo")
            .args(["ln", "-s", "/Users/Shared/.x", "/home/x"])
            .status()?;

        if !status.success() {
            eyre::bail!(
                "failed to create symlink /home/x. Please run:\n\
                 sudo ln -s /Users/Shared/.x /home/x"
            );
        }
    }

    // Create the store directory
    std::fs::create_dir_all(&store_dir)?;

    tracing::info!("Store created at {}", store_dir.display());
    Ok(())
}

/// Print setup instructions for the user
pub fn print_manual_setup_instructions() {
    #[cfg(target_os = "linux")]
    {
        tracing::info!("\n=== Manual Setup Instructions ===\n");
        tracing::info!("On Linux, run:");
        tracing::info!("  sudo mkdir -p /home/x");
        tracing::info!("  sudo chown $USER /home/x");
        tracing::info!("  mkdir -p /home/x/.x/store");
    }

    #[cfg(target_os = "macos")]
    {
        tracing::info!("\n=== Manual Setup Instructions ===\n");
        tracing::info!("On macOS, run:");
        tracing::info!("  mkdir -p /Users/Shared/.x/store");
        tracing::info!("  sudo ln -s /Users/Shared/.x /home/x");
        tracing::info!("");
        tracing::info!("If /home doesn't exist, first add to /etc/synthetic.conf:");
        tracing::info!("  echo 'home' | sudo tee -a /etc/synthetic.conf");
        tracing::info!("Then reboot and run the commands above.");
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        tracing::warn!("Manual setup instructions not available for this platform");
    }
}
