use crate::*;

#[derive(Parser)]
pub struct BackupArgs {
    /// Directory you want to back up
    #[arg(short, long, value_name = "DIR")]
    source_path: Option<PathBuf>,

    /// Set a custom backup destination
    #[arg(short, long, value_name = "DIR")]
    dest_path: Option<PathBuf>,

    /// Set a custom config file
    #[arg(short, long, value_name = "FILE")]
    config_path: Option<PathBuf>,
}

fn copy_dir_recursive(
    source_path: &Path,
    dest_path: &Path,
    pattern: &String,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest_path.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path, pattern)?;
        } else {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(entry.file_name().to_str().unwrap()) {
                fs::copy(&path, &dest_path)?;
                println!("Copied {:?} to {:?}", path, dest_path);
            }
        }
    }
    Ok(())
}

fn backup_files(
    source_path: &PathBuf,
    dest_path: &PathBuf,
    pattern: &String,
) -> Result<(), std::io::Error> {
    println!("Backing up files...");
    copy_dir_recursive(source_path, dest_path, pattern)?;
    println!("Backup completed successfully.");
    Ok(())
}

fn get_source_directory(
    source_path: &Option<PathBuf>,
    config: &mut Config,
) -> Result<PathBuf, std::io::Error> {
    if let Some(path) = source_path.as_deref() {
        println!("Source directory got at {}", path.display());

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
) -> Result<PathBuf, std::io::Error> {
    if let Some(path) = dest_path.as_deref() {
        println!("Destination directory got at {}", path.display());

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
            if !path.exists() {
                if let Err(err) = fs::create_dir_all(path) {
                    eprintln!("Cannot create {}: {}", path.display(), err);
                    return Err(err);
                }
                return Ok(path.to_path_buf());
            }
            eprintln!(
                "Destination directory {} is not a directory.",
                path.display()
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Destination directory invalid",
            ))
        }
    }
}

fn check_directories(
    config: &mut Config,
    source_path: &Option<PathBuf>,
    dest_path: &Option<PathBuf>,
) -> Result<(PathBuf, PathBuf), std::io::Error> {
    let s_path = get_source_directory(source_path, config)?;
    let d_path = get_dest_directory(dest_path, config)?;

    println!("Directories read successfully.");
    Ok((s_path, d_path))
}

pub fn command_backup(args: &BackupArgs) {
    let mut config = match config::get_config(&args.config_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    // update_config(&mut config, &args);

    let (source_path, dest_path) =
        match check_directories(&mut config, &args.source_path, &args.dest_path) {
            Ok((s, d)) => (s, d),
            Err(_) => {
                return;
            }
        };

    match backup_files(&source_path, &dest_path, &config.pattern) {
        Ok(_) => {}
        Err(_) => {
            return;
        }
    }
}
