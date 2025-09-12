use crate::*;

#[derive(Deserialize, Serialize)]
pub struct PathConfig {
    pub source_path: String,
    pub dest_path: String,
}

#[derive(Deserialize, Serialize)]
pub struct FileConfig {
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub date: Option<String>,
    pub size: Option<i64>,
    pub user: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub path_config: PathConfig,
    pub file_config: FileConfig,
}

#[derive(Parser)]
pub struct ConfigArgs {
    /// Set a custom config file
    #[arg(short, long, value_name = "FILE")]
    config_path: Option<PathBuf>,

    /// Directory you want to back up
    #[arg(short, long, value_name = "DIR")]
    source_path: Option<PathBuf>,

    /// Set a custom backup destination
    #[arg(short, long, value_name = "DIR")]
    dest_path: Option<PathBuf>,

    /// Set a custom file pattern
    #[arg(short, long, value_name = "REGEX")]
    file_name: Option<String>,

    /// Set a custom path
    #[arg(long, value_name = "PATH")]
    file_path: Option<String>,

    /// Set a custom date that the file was modified after
    #[arg(long, value_name = "DATE")]
    date: Option<String>,

    /// Set a custom size that the file is smaller than
    #[arg(long, value_name = "SIZE")]
    size: Option<i64>,

    /// Set a custom user
    #[arg(short, long, value_name = "USER")]
    user: Option<String>,

    /// Output config file content
    #[arg(short, long)]
    output: bool,

    /// Reset config to default
    #[arg(short, long)]
    reset: bool,
}

#[derive(Parser)]
pub struct ResetArgs {
    /// Reset a custom config file
    #[arg(short, long)]
    config_path: Option<PathBuf>,

    /// Directory you want to back up
    #[arg(short, long)]
    source_path: bool,

    /// Reset a custom backup destination
    #[arg(short, long)]
    dest_path: bool,

    /// Reset a custom file pattern
    #[arg(short, long)]
    file_name: bool,

    /// Reset a custom path
    #[arg(long)]
    file_path: bool,

    /// Reset a custom date that the file was modified after
    #[arg(long)]
    date: bool,

    /// Reset a custom size that the file is smaller than
    #[arg(long)]
    size: bool,

    /// Reset a custom user
    #[arg(short, long)]
    user: bool,

    /// Reset all configurations
    #[arg(short, long)]
    all: bool,
}

fn initialize_config(path: &Path) {
    let _ = std::fs::create_dir_all(path.parent().unwrap()).map_err(|err| {
        eprintln!(
            "Cannot create directory {}: {}",
            path.parent().unwrap().display(),
            err
        );
        err
    });

    let _ = File::create(&path).map_err(|err| {
        eprintln!("Cannot create {}: {}", path.display(), err);
        err
    });

    let path_config = PathConfig {
        source_path: dirs::home_dir()
            .unwrap()
            .join(".config/babycancer/source")
            .to_string_lossy()
            .to_string(),
        dest_path: dirs::home_dir()
            .unwrap()
            .join(".config/babycancer/dest")
            .to_string_lossy()
            .to_string(),
    };

    let file_config = FileConfig {
        file_path: None,
        file_name: None,
        date: None,
        size: None,
        user: None,
    };

    let config = Config {
        path_config,
        file_config,
    };

    update_config_file(path, &config);
}

fn check_config_file(config_path: &Option<PathBuf>) -> Result<PathBuf, std::io::Error> {
    if let Some(path) = config_path.as_deref() {
        match OpenOptions::new().read(true).write(true).open(path) {
            Ok(_) => Ok(path.to_path_buf()),
            Err(err) => {
                eprintln!("Cannot open {}: {}", path.display(), err);
                Err(err)
            }
        }
    } else {
        let path = dirs::home_dir()
            .unwrap()
            .join(".config/rustbackup/config.toml");

        match OpenOptions::new().read(true).write(true).open(path.clone()) {
            Ok(_) => {
                // Config file exists, read and parse it
                Ok(path)
            }
            Err(_) => {
                // Config file does not exist, create directories and initialize config
                initialize_config(&path);
                println!("Configuration file created at {}", path.display());
                Ok(path)
            }
        }
    }
}

fn update_config(config: &mut Config, args: &ConfigArgs) {
    let path_config = &mut config.path_config;
    let file_config = &mut config.file_config;

    if let Some(path) = args.source_path.as_deref() {
        path_config.source_path = path.to_string_lossy().to_string();
        println!("Source directory set to {}", path_config.source_path);
    }

    if let Some(path) = args.dest_path.as_deref() {
        path_config.dest_path = path.to_string_lossy().to_string();
        println!("Destination directory set to {}", path_config.dest_path);
    }

    if let Some(path) = args.file_path.as_deref() {
        file_config.file_path = Some(path.to_string());
        println!(
            "File path set to {}",
            file_config.file_path.as_ref().unwrap()
        );
    }

    if let Some(name) = args.file_name.as_deref() {
        file_config.file_name = Some(name.to_owned());
        println!(
            "File name set to {}",
            file_config.file_name.as_ref().unwrap()
        );
    }

    if let Some(date) = args.date.as_deref() {
        file_config.date = Some(date.to_owned());
        println!("Date set to {}", file_config.date.as_ref().unwrap());
    }

    if let Some(size) = args.size {
        file_config.size = Some(size);
        println!("Size set to {}", file_config.size.unwrap());
    }

    if let Some(user) = args.user.as_deref() {
        file_config.user = Some(user.to_owned());
        println!("User set to {}", file_config.user.as_ref().unwrap());
    }
}

fn update_config_file(path: &Path, config: &Config) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|err| {
            eprintln!("Cannot open {}: {}", path.display(), err);
            err
        })
        .unwrap();

    file.set_len(0).unwrap();

    let path_config = &config.path_config;
    let file_config = &config.file_config;
    let mut doc: DocumentMut = "".to_string().parse::<DocumentMut>().unwrap();

    let mut path_table = Table::new();
    path_table["source_path"] = Item::Value(path_config.source_path.clone().into());
    path_table["dest_path"] = Item::Value(path_config.dest_path.clone().into());
    doc["path_config"] = Item::Table(path_table);

    let mut file_table = Table::new();
    file_table["file_path"] = Item::Value(
        file_config
            .file_path
            .clone()
            .unwrap_or_else(|| "".to_string())
            .into(),
    );
    file_table["file_name"] = Item::Value(
        file_config
            .file_name
            .clone()
            .unwrap_or_else(|| "".to_string())
            .into(),
    );
    file_table["date"] = Item::Value(
        file_config
            .date
            .clone()
            .unwrap_or_else(|| "".to_string())
            .into(),
    );
    file_table["size"] = Item::Value(file_config.size.clone().unwrap_or(0).into());
    file_table["user"] = Item::Value(
        file_config
            .user
            .clone()
            .unwrap_or_else(|| "".to_string())
            .into(),
    );
    doc["file_config"] = Item::Table(file_table);

    file.write_all(doc.to_string().as_bytes()).unwrap();
}

fn read_config(path: &PathBuf) -> Result<Config, std::io::Error> {
    let content = fs::read_to_string(path).map_err(|err| {
        eprintln!("Failed to read {}: {}", path.display(), err);
        err
    })?;
    println!("Configuration file read at {}", path.display());

    let config: Config = toml::de::from_str(&content).map_err(|err| {
        eprintln!("Failed to parse config {:?}: {}", content, err);
        std::io::Error::new(std::io::ErrorKind::InvalidData, err)
    })?;

    Ok(config)
}

fn output_config(path: &PathBuf) -> Result<(), std::io::Error> {
    let content = fs::read_to_string(path).map_err(|err| {
        eprintln!("Failed to read {}: {}", path.display(), err);
        err
    })?;

    println!(
        "Configuration file read at {}:\n{}",
        path.display(),
        content
    );

    Ok(())
}

pub fn get_config(config_path: &Option<PathBuf>) -> Result<Config, std::io::Error> {
    let mut config_file = check_config_file(config_path)?;
    read_config(&mut config_file)
}

pub fn command_config(args: &ConfigArgs) -> Result<(), std::io::Error> {
    let config_path = check_config_file(&args.config_path)?;

    let mut config = read_config(&config_path)?;

    update_config(&mut config, &args);

    update_config_file(&config_path, &config);

    if args.output {
        output_config(&config_path)?;
    }

    Ok(())
}

fn reset_config(config: &mut Config, args: &ResetArgs) {
    let path_config = &mut config.path_config;
    let file_config = &mut config.file_config;

    if args.source_path || args.all {
        path_config.source_path = dirs::home_dir()
            .unwrap()
            .join(".config/babycancer/source")
            .to_string_lossy()
            .to_string();
        println!("Source directory reset to {}", path_config.source_path);
    }

    if args.dest_path || args.all {
        path_config.dest_path = dirs::home_dir()
            .unwrap()
            .join(".config/babycancer/dest")
            .to_string_lossy()
            .to_string();
        println!("Destination directory set to {}", path_config.dest_path);
    }

    if args.file_path || args.all {
        file_config.file_path = None;
        println!("File path reset");
    }

    if args.file_name || args.all {
        file_config.file_name = None;
        println!("File name reset");
    }

    if args.date || args.all {
        file_config.date = None;
        println!("Date reset");
    }

    if args.size || args.all {
        file_config.size = None;
        println!("Size reset");
    }

    if args.user || args.all {
        file_config.user = None;
        println!("User reset");
    }
}

pub fn command_reset(args: &ResetArgs) -> Result<(), std::io::Error> {
    let config_path = check_config_file(&args.config_path)?;

    let mut config = read_config(&config_path)?;

    reset_config(&mut config, &args);

    update_config_file(&config_path, &config);

    println!("Configuration file reset at {}", config_path.display());

    Ok(())
}
