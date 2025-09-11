use crate::*;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub source_path: String,
    pub dest_path: String,
    pub pattern: String,
}

#[derive(Parser)]
pub struct ConfigArgs {
    /// Directory you want to back up
    #[arg(short, long, value_name = "DIR")]
    source_path: Option<PathBuf>,

    /// Set a custom backup destination
    #[arg(short, long, value_name = "DIR")]
    dest_path: Option<PathBuf>,

    /// Set a custom config file
    #[arg(short, long, value_name = "FILE")]
    config_path: Option<PathBuf>,

    /// Set a custom file pattern
    #[arg(short, long, value_name = "REGEX")]
    pattern: Option<String>,

    /// Output config file content
    #[arg(short, long)]
    output: bool,
}

fn initialize_config(file: &mut File) {
    let config = Config {
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
        pattern: ".*".to_string(),
    };

    let toml_config = toml::to_string(&config).unwrap();
    file.write_all(toml_config.as_bytes()).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
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
                std::fs::create_dir_all(path.parent().unwrap()).map_err(|err| {
                    eprintln!(
                        "Cannot create directory {}: {}",
                        path.parent().unwrap().display(),
                        err
                    );
                    err
                })?;

                let mut file = File::create(&path).map_err(|err| {
                    eprintln!("Cannot create {}: {}", path.display(), err);
                    err
                })?;
                initialize_config(&mut file);
                println!("Configuration file created at {}", path.display());
                Ok(path)
            }
        }
    }
}

fn update_config(config: &mut Config, args: &ConfigArgs) {
    if let Some(path) = args.source_path.as_deref() {
        config.source_path = path.to_string_lossy().to_string();
        println!("Source directory set to {}", config.source_path);
    }
    if let Some(path) = args.dest_path.as_deref() {
        config.dest_path = path.to_string_lossy().to_string();
        println!("Destination directory set to {}", config.dest_path);
    }
    if let Some(pattern) = args.pattern.as_deref() {
        config.pattern = pattern.to_string();
        println!("File pattern set to {}", config.pattern);
    }
}

fn update_config_file(path: &Path, config: &Config) {
    let toml_config = toml::to_string(config).unwrap();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|err| {
            eprintln!("Cannot open {}: {}", path.display(), err);
            err
        }).unwrap();

    file.set_len(0).unwrap();
    file.write_all(toml_config.as_bytes()).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
}

fn read_config(path: &PathBuf) -> Result<Config, std::io::Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.clone())
        .map_err(|err| {
            eprintln!("Cannot open {}: {}", path.display(), err);
            err
        })?;
    let mut content = String::new();

    file.read_to_string(&mut content).map_err(|err| {
        eprintln!("Failed to read {:?}: {}", content, err);
        err
    })?;
    println!("Configuration file read at {}", path.display());
    file.seek(std::io::SeekFrom::Start(0)).unwrap();

    toml::from_str(&content).map_err(|err| {
        eprintln!("Failed to parse config {:?}: {}", content, err);
        std::io::Error::new(std::io::ErrorKind::InvalidData, err)
    })
}

fn output_config(path: &PathBuf) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.clone())
        .map_err(|err| {
            eprintln!("Cannot open {}: {}", path.display(), err);
            err
        }).unwrap();
    let mut content = String::new();

    file.read_to_string(&mut content).map_err(|err| {
        eprintln!("Failed to read {:?}: {}", content, err);
        err
    }).unwrap();
    println!("Configuration file at {}:\n{}", path.display(), content);
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
}

pub fn get_config(config_path: &Option<PathBuf>) -> Result<Config, std::io::Error> {
    let mut config_file = check_config_file(config_path)?;
    read_config(&mut config_file)
}

pub fn command_config(args: &ConfigArgs) {
    let config_path = match check_config_file(&args.config_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            return ;
        }
    };

    let mut config = match read_config(&config_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            return ;
        }
    };

    update_config(&mut config, &args);

    update_config_file(&config_path, &config);

    if args.output {
        output_config(&config_path);
    }
}
