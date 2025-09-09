use std::fs::{self, ReadDir};
use std::path::PathBuf;

use crate::config::Config;

fn get_source_directory(
    source_path: &Option<PathBuf>,
    config: &mut Config,
    edit_config: &bool,
) -> Result<ReadDir, std::io::Error> {
    if let Some(path) = source_path.as_deref() {
        println!("Source directory got at {}", path.display());
        if *edit_config {
            config.source_path = path.to_string_lossy().to_string();
        }

        fs::read_dir(path).map_err(|err| {
            eprintln!("Cannot read {}: {}", path.display(), err);
            err
        })
    } else {
        println!("Source directory got from config at {}", config.source_path);
        config
            .source_path
            .parse::<PathBuf>()
            .map_err(|err| {
                eprintln!("Invalid source path in config: {}", err);
                std::io::Error::new(std::io::ErrorKind::InvalidInput, err)
            })
            .and_then(|path| {
                fs::read_dir(&path).map_err(|err| {
                    eprintln!("Cannot read {}: {}", path.display(), err);
                    err
                })
            })
    }
}

fn get_dest_directory(
    dest_path: &Option<PathBuf>,
    config: &mut Config,
    edit_config: &bool,
) -> Result<ReadDir, std::io::Error> {
    if let Some(path) = dest_path.as_deref() {
        println!("Source directory got at {}", path.display());
        if *edit_config {
            config.dest_path = path.to_string_lossy().to_string();
        }

        match fs::read_dir(path) {
            Ok(entries) => {
                if entries.count() != 0 {
                    eprintln!(
                        "Warnning: Destination directory {} is not empty.",
                        path.display()
                    );
                }
                fs::read_dir(path)
            }
            Err(_) => {
                if let Err(err) = fs::create_dir_all(path) {
                    eprintln!("Cannot create {}: {}", path.display(), err);
                    return Err(err);
                }
                fs::read_dir(path)
            }
        }
    } else {
        println!(
            "Destination directory got from config at {}",
            config.dest_path
        );
        config
            .dest_path
            .parse::<PathBuf>()
            .map_err(|err| {
                eprintln!("Invalid source path in config: {}", err);
                std::io::Error::new(std::io::ErrorKind::InvalidInput, err)
            })
            .and_then(|path| {
                fs::read_dir(&path).map_err(|err| {
                    eprintln!("Cannot read {}: {}", path.display(), err);
                    err
                })
            })
    }
}

pub fn read_directories(
    config: &mut Config,
    source_path: &Option<PathBuf>,
    dest_path: &Option<PathBuf>,
    edit_config: &bool,
) -> Result<(ReadDir, ReadDir), std::io::Error> {
    let source_dir = get_source_directory(source_path, config, edit_config)?;

    let dest_dir = get_dest_directory(dest_path, config, edit_config)?;

    if *edit_config {
        let config_path = dirs::home_dir()
            .unwrap()
            .join(".config/rustbackup/config.toml");
        let toml_config = toml::to_string(config).unwrap();
        fs::write(&config_path, toml_config).map_err(|err| {
            eprintln!("Failed to write config {}: {}", config_path.display(), err);
            err
        })?;
        println!("Configuration file updated at {}", config_path.display());
    }
    Ok((dest_dir, source_dir))
}
