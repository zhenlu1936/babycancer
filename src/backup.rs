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

#[derive(Parser)]
pub struct TimerArgs {
    /// Interval in seconds to run the backup command
    #[arg(short, long, value_name = "SECONDS", default_value_t = 3600)]
    interval: u64,

    /// Set a custom config file
    #[arg(short, long, value_name = "FILE")]
    config_path: Option<PathBuf>,
}

#[derive(Parser)]
pub struct RealtimeArgs {
    /// Set a custom config file
    #[arg(short, long, value_name = "FILE")]
    config_path: Option<PathBuf>,
}

fn check_file_properties(
    root_path: &Path,
    file_path: &Path,
    file_config: &config::FileConfig,
) -> bool {
    let metadata = match fs::metadata(&file_path) {
        Ok(metadata) => metadata,
        Err(_) => {
            eprintln!("Failed to get metadata for {:?}", &file_path);
            return false;
        }
    };

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
            let file_date = metadata.modified().unwrap();
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
            let file_size = metadata.len() as i64;
            if file_size < size {
                return false;
            }
        }
    }

    if let Some(ref user) = file_config.user {
        if !user.is_empty() {
            let owner = metadata.uid();
            if owner.to_string() != *user {
                return false;
            }
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
                let metadata = match fs::metadata(&entry_path) {
                    Ok(metadata) => metadata,
                    Err(_) => {
                        eprintln!("Failed to get metadata for {:?}", &entry_path);
                        continue;
                    }
                };
                if metadata.file_type().is_symlink() {
                    eprintln!("Skipping symbolic link: {:?}", &entry_path);
                } else {
                    match fs::copy(&entry_path, &dest_path) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!(
                                "Failed to copy {:?} to {:?}: {}",
                                &entry_path, &dest_path, e
                            );
                            continue;
                        }
                    }
                }
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

pub fn command_backup(args: &BackupArgs) -> Result<(), std::io::Error> {
    let mut config = config::get_config(&args.config_path)?;

    let (source_path, dest_path) =
        check_directories(&mut config, &args.source_path, &args.dest_path)?;

    backup_files(&source_path, &dest_path, &config.file_config)?;

    Ok(())
}

pub fn command_timer(args: &TimerArgs) -> Result<(), std::io::Error> {
    let interval = args.interval;
    if interval == 0 {
        eprintln!("Interval must be greater than 0.");
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Interval must be greater than 0",
        ));
    }

    println!("Starting timer with interval of {} seconds...", interval);
    loop {
        let start = std::time::Instant::now();
        println!("Running timer backup...");

        let mut config = config::get_config(&args.config_path)?;

        let (source_path, dest_path) = check_directories(&mut config, &None, &None)?;

        if let Err(err) = backup_files(&source_path, &dest_path, &config.file_config) {
            eprintln!("Backup command failed: {}", err);
        }

        let elapsed = start.elapsed();
        if elapsed.as_secs() < interval {
            let sleep_duration = std::time::Duration::from_secs(interval - elapsed.as_secs());
            std::thread::sleep(sleep_duration);
        }
    }
}

pub fn command_realtime(args: &RealtimeArgs) -> Result<(), std::io::Error> {
    let mut config = config::get_config(&args.config_path)?;

    let (source_path, dest_path) = check_directories(&mut config, &None, &None)?;

    println!("Starting real-time backup...");

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                tx.send(event).unwrap();
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        })
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    watcher
        .watch(&source_path, notify::RecursiveMode::Recursive)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("Change detected: {:?}", event);
                if let Err(err) = backup_files(&source_path, &dest_path, &config.file_config) {
                    eprintln!("Backup command failed: {}", err);
                }
            }
            Err(e) => eprintln!("recv error: {:?}", e),
        }
    }
}
