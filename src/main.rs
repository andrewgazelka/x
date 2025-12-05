mod bootstrap;
mod exec;
mod fetch;
mod manifest;
mod nar;
mod ref_parser;
mod store;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "x", about = "Rootless Nix package runner", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
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

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { package, args } => cmd_run(&package, &args),
        Commands::Info { package } => cmd_info(&package),
        Commands::List => cmd_list(),
        Commands::Setup => cmd_setup(),
    }
}

fn cmd_run(package: &str, args: &[String]) -> eyre::Result<()> {
    // Ensure store is set up
    bootstrap::ensure_store()?;

    // Parse the package reference
    let pkg_ref = ref_parser::parse(package)?;

    // Fetch manifest
    let manifest = fetch::fetch_manifest(&pkg_ref)?;

    // Get the requested output
    let output = manifest.get_output(&pkg_ref.output)?;

    // Ensure all dependencies are fetched
    fetch::ensure_closure(output)?;

    // Run the binary
    exec::run(output, args)
}

fn cmd_info(package: &str) -> eyre::Result<()> {
    let pkg_ref = ref_parser::parse(package)?;
    let manifest = fetch::fetch_manifest(&pkg_ref)?;

    println!("Package: {}", manifest.repository);
    println!("Ref: {}", manifest.git_ref);
    println!("Commit: {}", manifest.commit);
    println!("Built: {}", manifest.built_at);
    println!("System: {}", manifest.system);
    println!();
    println!("Outputs:");

    for (name, output) in &manifest.outputs {
        let bin_info = output
            .bin
            .as_ref()
            .map(|b| format!(" (bin: {b})"))
            .unwrap_or_default();
        println!(
            "  {name}: {} ({} deps){bin_info}",
            output.store_path,
            output.closure.len()
        );
    }

    Ok(())
}

fn cmd_list() -> eyre::Result<()> {
    let store_dir = store::store_dir();

    if !store_dir.exists() {
        println!("No packages installed (store not initialized)");
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(&store_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .collect();

    if entries.is_empty() {
        println!("No packages installed");
        return Ok(());
    }

    println!("Installed store paths ({}):", entries.len());
    for entry in entries {
        println!("  {}", entry.file_name().to_string_lossy());
    }

    Ok(())
}

fn cmd_setup() -> eyre::Result<()> {
    bootstrap::print_manual_setup_instructions();
    Ok(())
}
