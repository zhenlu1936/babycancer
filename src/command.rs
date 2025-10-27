use crate::*;

// Rust program to backup files in your directories
#[derive(Parser)]
#[command(name = "babycancer")]
#[command(version = "0.4")]
#[command(override_usage = "[COMMAND] [OPTIONS]")]
#[command(about = "Backup files in your directories", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Exit the program
    Exit,

    /// Backup files
    Backup(backup::BackupArgs),

    /// Edit configuration file
    Config(config::ConfigArgs),

    /// Reset configuration file to default values
    Reset(config::ResetArgs),
}

pub fn execute_command(args: Args) -> Result<(), io::Error> {
    match &args.command {
        Some(Commands::Exit) => {
            println!("Exiting the program.");
            std::process::exit(0);
        }

        Some(Commands::Backup(args)) => backup::command_backup(args),

        Some(Commands::Config(args)) => config::command_config(args),

        Some(Commands::Reset(args)) => config::command_reset(args),

        None => {
            println!("No command provided. Use --help for more information.");
            Ok(())
        }
    }
}

pub fn get_args(line: String) -> Result<Args, clap::Error> {
    let tokens = line.split_whitespace();
    let argv = std::iter::once("myprog").chain(tokens);
    let args = match Args::try_parse_from(argv) {
        Ok(a) => a,
        Err(e) => {
            return Err(e);
        }
    };
    Ok(args)
}
