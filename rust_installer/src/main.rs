use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Install Vim and related assets
    Install,
    /// Uninstall Vim and remove installed assets
    Uninstall,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Install) => println!("Running Rust installer (placeholder)"),
        Some(Command::Uninstall) => println!("Running Rust uninstaller (placeholder)"),
        None => println!("Specify 'install' or 'uninstall'"),
    }
}
