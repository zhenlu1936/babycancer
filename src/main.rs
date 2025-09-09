use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod config;
mod directory;
// Rust program to backup files in your directories
#[derive(Parser)]
#[command(name = "RustBackup")]
#[command(version = "0.1")]
#[command(about = "Backup files in your directories", long_about = None)]
struct Cli {
    /// Directory you want to back up
    #[arg(short, long, value_name = "DIR")]
    source_path: Option<PathBuf>,

    /// Sets a custom backup destination
    #[arg(short, long, value_name = "DIR")]
    dest_path: Option<PathBuf>,

    // /// Sets a custom config file
    // #[arg(short, long, value_name = "DIR")]
    // config_path: Option<PathBuf>,
    /// Apply edition to config in command to config file
    #[arg(short, long)]
    edit_config: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Debug {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

fn backup_files() {
    // Placeholder for backup logic
    println!("Backing up files...");
}

fn main() {
    let cli = Cli::parse();

    let mut config = match config::read_config() {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let (_source_dir, _dest_dir) = match directory::read_directories(
        &mut config,
        &cli.source_path,
        &cli.dest_path,
        &cli.edit_config,
    ) {
        Ok((s, d)) => (s, d),
        Err(_) => {
            std::process::exit(1);
        }
    };

    backup_files();

    match &cli.command {
        Some(Commands::Debug { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {}
    }
}
