use crate::*;

#[derive(Parser)]
pub struct BackupArgs {
    /// Set a custom config file
    #[arg(short, long, value_name = "FILE")]
    config_path: Option<PathBuf>,

    /// Backup files in scheduled intervals
    #[arg(short, long)]
    interval: Option<u64>,

    /// Backup files in real-time when changes are detected
    #[arg(short, long)]
    realtime: bool,
}

fn check_file_properties(
    root_path: &Path,
    file_path: &Path,
    file_config: &config::FileConfig,
) -> bool {
    let metadata = match fs::symlink_metadata(&file_path) {
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

    if let Some(ref user_name) = file_config.user {
        if !user_name.is_empty() {
            let owner_uid = metadata.uid();
            let owner_name =
                get_user_by_uid(owner_uid).map(|u| u.name().to_string_lossy().into_owned());
            if owner_name != Some(user_name.clone()) {
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
    file_config: &config::FileConfig,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest_path.join(entry.file_name());

        if entry_path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&root_path, &entry_path, &dest_path, file_config)?;
            if fs::read_dir(&dest_path)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(false)
            {
                fs::remove_dir(dest_path)?;
            }
        } else {
            if check_file_properties(&root_path, &entry_path, file_config) {
                let metadata = match fs::symlink_metadata(&entry_path) {
                    Ok(metadata) => metadata,
                    Err(_) => {
                        eprintln!("Failed to get metadata for {:?}", &entry_path);
                        continue;
                    }
                };
                if metadata.file_type().is_symlink() {
                    // xxx: cannot remove symlink?
                    let target = fs::read_link(&entry_path)?;
                    if dest_path.exists() {
                        fs::remove_file(&dest_path)?;
                    }
                    if let Err(e) = std::os::unix::fs::symlink(&target, &dest_path) {
                        eprintln!(
                            "Failed to create symlink from {:?} to {:?}: {}",
                            &target, &dest_path, e
                        );
                        continue;
                    }
                    println!("Copied symlink {:?} to {:?}", &entry_path, &dest_path);
                } else if metadata.file_type().is_fifo() {
                    if dest_path.exists() {
                        fs::remove_file(&dest_path)?;
                    }
                    if let Err(e) = nix::unistd::mkfifo(&dest_path, nix::sys::stat::Mode::S_IRWXU) {
                        eprintln!("Failed to create FIFO {:?}: {}", &dest_path, e);
                        continue;
                    }
                    println!("Copied FIFO {:?} to {:?}", &entry_path, &dest_path);
                } else if metadata.file_type().is_char_device() {
                    if dest_path.exists() {
                        fs::remove_file(&dest_path)?;
                    }
                    mknod(
                        &dest_path,
                        nix::sys::stat::SFlag::S_IFCHR,
                        nix::sys::stat::Mode::from_bits_truncate(
                            metadata.mode().try_into().unwrap(),
                        ),
                        metadata.rdev() as i32,
                    )?;
                    println!("Copied char device {:?} to {:?}", &entry_path, &dest_path);
                } else if metadata.file_type().is_block_device() {
                    if dest_path.exists() {
                        fs::remove_file(&dest_path)?;
                    }
                    mknod(
                        &dest_path,
                        nix::sys::stat::SFlag::S_IFBLK,
                        nix::sys::stat::Mode::from_bits_truncate(
                            metadata.mode().try_into().unwrap(),
                        ),
                        metadata.rdev() as i32,
                    )?;
                    println!("Copied block device {:?} to {:?}", &entry_path, &dest_path);
                } else {
                    match fs::copy(&entry_path, &dest_path) {
                        Ok(_) => {
                            println!("Copied {:?} to {:?}", &entry_path, &dest_path);
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to copy {:?} to {:?}: {}",
                                &entry_path, &dest_path, e
                            );
                            continue;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn backup_files(
    source_path: &PathBuf,
    dest_path: &PathBuf,
    file_config: &config::FileConfig,
    output_config: &config::OutputConfig,
) -> Result<(), std::io::Error> {
    println!("Backing up files...");
    if output_config.tar {
        let tar_path = dest_path.join("backup.tar");
        let tar_file = File::create(&tar_path)?;
        let mut tar_builder = tar::Builder::new(tar_file);
        tar_builder.append_dir_all(".", source_path)?;
        tar_builder.finish()?;
        println!("Created tar archive at {:?}", tar_path);
        return Ok(());
    } else {
        copy_dir_recursive(source_path, source_path, dest_path, file_config)?;
    }
    println!("Backup completed successfully.");
    Ok(())
}

fn get_source_directory(config: &mut Config) -> Result<PathBuf, std::io::Error> {
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

fn get_dest_directory(config: &mut Config) -> Result<PathBuf, std::io::Error> {
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

fn check_directories(config: &mut Config) -> Result<(PathBuf, PathBuf), std::io::Error> {
    let source_path = get_source_directory(config)?;
    let dest_path = get_dest_directory(config)?;

    println!("Directories read successfully.");
    Ok((source_path, dest_path))
}

fn timed_backup(
    source_path: &PathBuf,
    dest_path: &PathBuf,
    file_config: &config::FileConfig,
    output_config: &config::OutputConfig,
    interval: u64,
) -> Result<(), std::io::Error> {
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

        if let Err(err) = backup_files(&source_path, &dest_path, file_config, output_config) {
            eprintln!("Backup command failed: {}", err);
        }

        let elapsed = start.elapsed();
        if elapsed.as_secs() < interval {
            let sleep_duration = std::time::Duration::from_secs(interval - elapsed.as_secs());
            std::thread::sleep(sleep_duration);
        }
    }
}

fn realtime_backup(
    source_path: &PathBuf,
    dest_path: &PathBuf,
    file_config: &config::FileConfig,
    output_config: &config::OutputConfig,
) -> Result<(), std::io::Error> {
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
                if let Err(err) = backup_files(&source_path, &dest_path, file_config, output_config)
                {
                    eprintln!("Backup command failed: {}", err);
                }
            }
            Err(e) => eprintln!("recv error: {:?}", e),
        }
    }
}

pub fn command_backup(args: &BackupArgs) -> Result<(), std::io::Error> {
    let mut config = config::get_config(&args.config_path)?;
    let (source_path, dest_path) = check_directories(&mut config)?;
    let file_config = &config.file_config;
    let output_config = &config.output_config;

    if args.realtime {
        realtime_backup(&source_path, &dest_path, file_config, output_config)
    } else if let Some(interval) = args.interval {
        timed_backup(
            &source_path,
            &dest_path,
            file_config,
            output_config,
            interval,
        )
    } else {
        backup_files(&source_path, &dest_path, file_config, output_config)
    }
}
