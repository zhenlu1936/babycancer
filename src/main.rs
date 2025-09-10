use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod backup;
mod config;
mod directory;

// Rust program to backup files in your directories
#[derive(Parser)]
#[command(name = "RustBackup")]
#[command(version = "0.2")]
#[command(about = "Backup files in your directories", long_about = None)]
struct Cli {
    /// Directory you want to back up
    #[arg(short, long, value_name = "DIR")]
    source_path: Option<PathBuf>,

    /// Sets a custom backup destination
    #[arg(short, long, value_name = "DIR")]
    dest_path: Option<PathBuf>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config_path: Option<PathBuf>,

    /// Update config file
    #[arg(short, long)]
    update_config: bool,

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

fn main() {
    let cli = Cli::parse();

    let mut config_file = match config::check_config_file(&cli.config_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let mut config = match config::read_config(&mut config_file) {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let (source_path, dest_path) = match directory::check_directories(
        &mut config,
        &cli.source_path,
        &cli.dest_path,
        &cli.update_config,
    ) {
        Ok((s, d)) => (s, d),
        Err(_) => {
            std::process::exit(1);
        }
    };

    match backup::backup_files(&source_path, &dest_path) {
        Ok(_) => {}
        Err(_) => {
            std::process::exit(1);
        }
    }

    if cli.update_config {
        config::update_config_file(&mut config_file, &config);
    }

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
