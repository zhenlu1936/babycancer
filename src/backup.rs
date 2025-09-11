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

fn check_file_properties(
    root_path: &Path,
    file_path: &Path,
    file_config: &config::FileConfig,
) -> bool {
    if let Some(ref config_path) = file_config.file_path {
        if !(file_path).starts_with(root_path.join(config_path)) {
            return false;
        }
    }

    if let Some(ref name) = file_config.file_name {
        let re = Regex::new(name).unwrap();
        if !re.is_match(file_path.file_name().unwrap().to_str().unwrap()) {
            return false;
        }
    }

    if let Some(ref date) = file_config.date {
        if !date.is_empty() {
            let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
            let file_date = file_path.metadata().unwrap().modified().unwrap();
            let file_date = chrono::DateTime::<chrono::Local>::from(file_date)
                .naive_local()
                .date();
            if file_date != date {
                return false;
            }
        }
    }

    if let Some(size) = file_config.size {
        if size != 0 {
            let file_size = file_path.metadata().unwrap().len() as i64;
            if file_size < size {
                return false;
            }
        }
    }

    if let Some(ref user) = file_config.user {
        let Ok(metadata) = fs::metadata(file_path) else {
            return false;
        };
        let owner = metadata.uid();
        if owner.to_string() != *user {
            return false;
        } else {
            return false;
        }
    }

    true
}

fn copy_dir_recursive(
    root_path: &Path,
    source_path: &Path,
    dest_path: &Path,
    file_properties: &config::FileConfig,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest_path.join(entry.file_name());

        if entry_path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&root_path, &entry_path, &dest_path, file_properties)?;
            if fs::read_dir(&dest_path)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(false)
            {
                fs::remove_dir(dest_path)?;
            }
        } else {
            if check_file_properties(&root_path, &entry_path, file_properties) {
                fs::copy(&entry_path, &dest_path)?;
                println!("Copied {:?} to {:?}", &entry_path, &dest_path);
            }
        }
    }
    Ok(())
}

fn backup_files(
    source_path: &PathBuf,
    dest_path: &PathBuf,
    file_properties: &config::FileConfig,
) -> Result<(), std::io::Error> {
    println!("Backing up files...");
    copy_dir_recursive(source_path, source_path, dest_path, file_properties)?;
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
        println!(
            "Source directory got from config at {}",
            config.path_config.source_path
        );
        let path = Path::new(&config.path_config.source_path);
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
            config.path_config.dest_path
        );
        let path = Path::new(&config.path_config.dest_path);
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

    let (source_path, dest_path) =
        match check_directories(&mut config, &args.source_path, &args.dest_path) {
            Ok((s, d)) => (s, d),
            Err(_) => {
                return;
            }
        };

    match backup_files(&source_path, &dest_path, &config.file_config) {
        Ok(_) => {}
        Err(_) => {
            return;
        }
    }
}
