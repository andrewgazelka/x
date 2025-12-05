fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { package, args } => libx::run(&package, &args),
        Commands::Info { package } => cmd_info(&package),
        Commands::List => cmd_list(),
        Commands::Setup => {
            cmd_setup();
            Ok(())
        }
    }
}

#[derive(clap::Parser)]
#[command(name = "x", about = "Rootless Nix package runner", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run a package
    Run {
        /// Package reference (github:owner/repo/ref#output)
        package: String,

        /// Arguments to pass to the binary
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Show package information
    Info {
        /// Package reference (github:owner/repo/ref#output)
        package: String,
    },

    /// List installed packages
    List,

    /// Show setup instructions
    Setup,
}

use clap::Parser;

fn cmd_info(package: &str) -> eyre::Result<()> {
    let info = libx::info(package)?;

    tracing::info!(repository = %info.repository, "Package");
    tracing::info!(git_ref = %info.git_ref, "Ref");
    tracing::info!(commit = %info.commit, "Commit");
    tracing::info!(built_at = %info.built_at, "Built");
    tracing::info!(system = %info.system, "System");
    tracing::info!("Outputs:");

    for output in &info.outputs {
        let bin_info = output
            .bin
            .as_ref()
            .map(|b| format!(" (bin: {b})"))
            .unwrap_or_default();
        tracing::info!(
            "  {}: {} ({} deps){bin_info}",
            output.name,
            output.store_path,
            output.closure_size
        );
    }

    Ok(())
}

fn cmd_list() -> eyre::Result<()> {
    let entries = libx::list()?;

    if entries.is_empty() {
        tracing::info!("No packages installed");
        return Ok(());
    }

    tracing::info!("Installed store paths ({}):", entries.len());
    for entry in entries {
        tracing::info!("  {entry}");
    }

    Ok(())
}

fn cmd_setup() {
    libx::bootstrap::print_manual_setup_instructions();
}
