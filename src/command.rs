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

    /// Scheduled backup
    TimedBackup(backup::TimedBackupArgs),

    /// Real-time backup
    RealtimeBackup(backup::RealtimeBackupArgs),
}
