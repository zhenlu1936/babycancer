use std::fs::{self};
use std::path::{Path, PathBuf};

use crate::config::Config;

fn get_source_directory(
    source_path: &Option<PathBuf>,
    config: &mut Config,
    edit_config: &bool,
) -> Result<PathBuf, std::io::Error> {
    if let Some(path) = source_path.as_deref() {
        println!("Source directory got at {}", path.display());
        if *edit_config {
            config.source_path = path.to_string_lossy().to_string();
        }

        if path.exists() && path.is_dir() {
            Ok(path.to_path_buf())
        } else {
            eprintln!(
                "Source directory {} does not exist or is not a directory.",
                path.display()
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Source directory not found or invalid",
            ))
        }
    } else {
        println!("Source directory got from config at {}", config.source_path);
        let path = Path::new(&config.source_path);
        if path.exists() && path.is_dir() {
            Ok(path.to_path_buf())
        } else {
            eprintln!(
                "Source directory {} does not exist or is not a directory.",
                path.display()
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Source directory not found or invalid",
            ))
        }
    }
}

fn get_dest_directory(
    dest_path: &Option<PathBuf>,
    config: &mut Config,
    edit_config: &bool,
) -> Result<PathBuf, std::io::Error> {
    if let Some(path) = dest_path.as_deref() {
        println!("Destination directory got at {}", path.display());
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
                Ok(path.to_path_buf())
            }
            Err(_) => {
                if let Err(err) = fs::create_dir_all(path) {
                    eprintln!("Cannot create {}: {}", path.display(), err);
                    return Err(err);
                }
                Ok(path.to_path_buf())
            }
        }
    } else {
        println!(
            "Destination directory got from config at {}",
            config.dest_path
        );
        let path = Path::new(&config.dest_path);
        if path.exists() && path.is_dir() {
            Ok(path.to_path_buf())
        } else {
            eprintln!(
                "Destination directory {} does not exist or is not a directory.",
                path.display()
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Destination directory not found or invalid",
            ))
        }
    }
}

pub fn check_directories(
    config: &mut Config,
    source_path: &Option<PathBuf>,
    dest_path: &Option<PathBuf>,
    edit_config: &bool,
) -> Result<(PathBuf, PathBuf), std::io::Error> {
    let s_path = get_source_directory(source_path, config, edit_config)?;
    let d_path = get_dest_directory(dest_path, config, edit_config)?;

    println!("Directories read successfully.");
    Ok((s_path, d_path))
}
