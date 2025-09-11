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
        pattern: "*".to_string(),
    };

    let toml_config = toml::to_string(&config).unwrap();
    file.write_all(toml_config.as_bytes()).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
}

fn read_config_file(config_path: &Option<PathBuf>) -> Result<File, std::io::Error> {
    if let Some(path) = config_path.as_deref() {
        match OpenOptions::new().read(true).write(true).open(path) {
            Ok(file) => {
                println!("Configuration file read at {}", path.display());
                Ok(file)
            }
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
            Ok(file) => {
                // Config file exists, read and parse it
                println!("Configuration file read at {}", path.display());
                Ok(file)
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
                Ok(file)
            }
        }
    }
}

fn update_config(config: &mut Config, args: &ConfigArgs) {
    if let Some(path) = args.source_path.as_deref() {
        config.source_path = path.to_string_lossy().to_string();
    }
    if let Some(path) = args.dest_path.as_deref() {
        config.dest_path = path.to_string_lossy().to_string();
    }
    if let Some(pattern) = args.pattern.as_deref() {
        config.pattern = pattern.to_string();
    }
}

fn update_config_file(config_file: &mut File, config: &Config) {
    let toml_config = toml::to_string(config).unwrap();
    config_file.set_len(0).unwrap();
    config_file.write_all(toml_config.as_bytes()).unwrap();
    config_file.seek(std::io::SeekFrom::Start(0)).unwrap();
}

fn read_config(config_file: &mut File) -> Result<Config, std::io::Error> {
    let mut content = String::new();
    config_file.read_to_string(&mut content).map_err(|err| {
        eprintln!("Failed to read {:?}: {}", content, err);
        err
    })?;
    config_file.seek(std::io::SeekFrom::Start(0)).unwrap();

    toml::from_str(&content).map_err(|err| {
        eprintln!("Failed to parse config {:?}: {}", content, err);
        std::io::Error::new(std::io::ErrorKind::InvalidData, err)
    })
}

pub fn get_config(config_path: &Option<PathBuf>) -> Result<Config, std::io::Error> {
    let mut config_file = read_config_file(config_path)?;
    read_config(&mut config_file)
}

pub fn command_config(args: &ConfigArgs) {
    let mut config_file = match read_config_file(&args.config_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let mut config = match read_config(&mut config_file) {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    update_config(&mut config, &args);

    update_config_file(&mut config_file, &config);
}
