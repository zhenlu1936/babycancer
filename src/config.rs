use crate::*;
use std::sync::{Mutex, OnceLock};

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
pub struct OutputConfig {
    pub tar: bool,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub path_config: PathConfig,
    pub file_config: FileConfig,
    pub output_config: OutputConfig,
}

// In-process retention for the last used config path. This lets commands like
// `backup` reuse a path previously set via `config -c <path>` in the same REPL/session.
static CURRENT_CONFIG_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

fn config_path_cell() -> &'static Mutex<Option<PathBuf>> {
    CURRENT_CONFIG_PATH.get_or_init(|| Mutex::new(None))
}

fn set_current_config_path(path: PathBuf) {
    let mut guard = config_path_cell().lock().unwrap();
    *guard = Some(path);
}

fn get_current_config_path() -> Option<PathBuf> {
    let guard = config_path_cell().lock().unwrap();
    guard.clone()
}

trait ValidConfig {
    fn initialize() -> Self;
    fn table(&self) -> Table;
    fn update(&mut self, args: &ConfigArgs);
    fn reset(&mut self, args: &ResetArgs);
}

impl ValidConfig for PathConfig {
    fn initialize() -> Self {
        PathConfig {
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
        }
    }

    fn table(&self) -> Table {
        let mut table = Table::new();
        table["source_path"] = Item::Value(self.source_path.clone().into());
        table["dest_path"] = Item::Value(self.dest_path.clone().into());
        table
    }

    fn update(&mut self, args: &ConfigArgs) {
        if let Some(path) = args.source_path.as_deref() {
            self.source_path = path.to_string_lossy().to_string();
            println!("Source directory set to {}", self.source_path);
        }

        if let Some(path) = args.dest_path.as_deref() {
            self.dest_path = path.to_string_lossy().to_string();
            println!("Destination directory set to {}", self.dest_path);
        }
    }

    fn reset(&mut self, args: &ResetArgs) {
        if args.source_path || args.all {
            self.source_path = dirs::home_dir()
                .unwrap()
                .join(".config/babycancer/source")
                .to_string_lossy()
                .to_string();
            println!("Source directory reset to {}", self.source_path);
        }

        if args.dest_path || args.all {
            self.dest_path = dirs::home_dir()
                .unwrap()
                .join(".config/babycancer/dest")
                .to_string_lossy()
                .to_string();
            println!("Destination directory set to {}", self.dest_path);
        }
    }
}

impl ValidConfig for FileConfig {
    fn initialize() -> Self {
        FileConfig {
            file_path: None,
            file_name: None,
            date: None,
            size: None,
            user: None,
        }
    }

    fn table(&self) -> Table {
        let mut table = Table::new();
        table["file_path"] = match &self.file_path {
            Some(path) => Item::Value(path.clone().into()),
            None => Item::None,
        };
        table["file_name"] = match &self.file_name {
            Some(name) => Item::Value(name.clone().into()),
            None => Item::None,
        };
        table["date"] = match &self.date {
            Some(date) => Item::Value(date.clone().into()),
            None => Item::None,
        };
        table["size"] = match self.size {
            Some(size) => Item::Value(size.into()),
            None => Item::None,
        };
        table["user"] = match &self.user {
            Some(user) => Item::Value(user.clone().into()),
            None => Item::None,
        };
        table
    }

    fn update(&mut self, args: &ConfigArgs) {
        if let Some(path) = args.file_path.as_deref() {
            self.file_path = Some(path.to_string());
            println!(
                "File path set to {}",
                self.file_path.as_ref().unwrap()
            );
        }

        if let Some(name) = args.file_name.as_deref() {
            self.file_name = Some(name.to_owned());
            println!(
                "File name set to {}",
                self.file_name.as_ref().unwrap()
            );
        }

        if let Some(date) = args.date.as_deref() {
            self.date = Some(date.to_owned());
            println!("Date set to {}", self.date.as_ref().unwrap());
        }

        if let Some(size) = args.size {
            self.size = Some(size);
            println!("Size set to {}", self.size.unwrap());
        }

        if let Some(user) = args.user.as_deref() {
            self.user = Some(user.to_owned());
            println!("User set to {}", self.user.as_ref().unwrap());
        }
    }

    fn reset(&mut self, args: &ResetArgs) {
        if args.file_path || args.all {
            self.file_path = None;
            println!("File path reset");
        }

        if args.file_name || args.all {
            self.file_name = None;
            println!("File name reset");
        }

        if args.date || args.all {
            self.date = None;
            println!("Date reset");
        }

        if args.size || args.all {
            self.size = None;
            println!("Size reset");
        }

        if args.user || args.all {
            self.user = None;
            println!("User reset");
        }
    }
}

impl ValidConfig for OutputConfig {
    fn initialize() -> Self {
        OutputConfig { tar: false }
    }

    fn table(&self) -> Table {
        let mut table = Table::new();
        table["tar"] = Item::Value(self.tar.into());
        table
    }
    
    fn update(&mut self, args: &ConfigArgs) {
        if let Some(tar) = args.tar {
            self.tar = tar;
            println!("Use tar for backup: {}", self.tar);
        }
    }

    fn reset(&mut self, args: &ResetArgs) {
        if args.tar || args.all {
            self.tar = false;
            println!("Use tar reset");
        }
    }
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

    /// Use tar for backup
    #[arg(short, long)]
    tar: Option<bool>,

    /// Output config file content
    #[arg(short, long)]
    output: bool,
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

    /// Use tar for backup
    #[arg(short, long)]
    tar: bool,

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

    let path_config = PathConfig::initialize();
    let file_config = FileConfig::initialize();
    let output_config = OutputConfig::initialize();

    let config = Config {
        path_config,
        file_config,
        output_config,
    };

    update_config_file(path, &config);
}

fn check_config_file(config_path: &Option<PathBuf>) -> Result<PathBuf, std::io::Error> {
    // Precedence:
    // 1) explicit path from args
    // 2) last path set in this process via set_current_config_path
    // 3) default path under ~/.config/rustbackup/config.toml
    let chosen: Option<PathBuf> = match config_path {
        Some(p) => Some(p.clone()),
        None => get_current_config_path(),
    };

    if let Some(path) = chosen {
        match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(_) => return Ok(path),
            Err(err) => {
                eprintln!("Cannot open {}: {}", path.display(), err);
                return Err(err);
            }
        }
    }

    // Fallback to default path
    let path = dirs::home_dir()
        .unwrap()
        .join(".config/rustbackup/config.toml");

    match OpenOptions::new().read(true).write(true).open(path.clone()) {
        Ok(_) => {
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

fn update_config(config: &mut Config, args: &ConfigArgs) {
    let path_config = &mut config.path_config;
    let file_config = &mut config.file_config;
    let output_config = &mut config.output_config;

    path_config.update(args);
    file_config.update(args);
    output_config.update(args);
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
    let output_config = &config.output_config;

    let mut doc: DocumentMut = "".to_string().parse::<DocumentMut>().unwrap();

    doc["path_config"] = Item::Table(path_config.table());
    doc["file_config"] = Item::Table(file_config.table());
    doc["output_config"] = Item::Table(output_config.table());

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

fn print_config(path: &PathBuf) -> Result<(), std::io::Error> {
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

fn reset_config(config: &mut Config, args: &ResetArgs) {
    let path_config = &mut config.path_config;
    let file_config = &mut config.file_config;
    let output_config = &mut config.output_config;

    path_config.reset(args);
    file_config.reset(args);
    output_config.reset(args);
}

pub fn get_config(config_path: &Option<PathBuf>) -> Result<Config, std::io::Error> {
    let mut config_file = check_config_file(config_path)?;
    read_config(&mut config_file)
}

pub fn command_config(args: &ConfigArgs) -> Result<(), std::io::Error> {
    let config_path = check_config_file(&args.config_path)?;

    // If the user explicitly provided a config path, retain it for this process.
    if let Some(p) = &args.config_path {
        set_current_config_path(p.clone());
    }

    let mut config = read_config(&config_path)?;

    update_config(&mut config, &args);

    update_config_file(&config_path, &config);

    if args.output {
        print_config(&config_path)?;
    }

    Ok(())
}

pub fn command_reset(args: &ResetArgs) -> Result<(), std::io::Error> {
    let config_path = check_config_file(&args.config_path)?;

    // If the user explicitly provided a config path, retain it for this process.
    if let Some(p) = &args.config_path {
        set_current_config_path(p.clone());
    }

    let mut config = read_config(&config_path)?;

    reset_config(&mut config, &args);

    update_config_file(&config_path, &config);

    println!("Configuration file reset at {}", config_path.display());

    Ok(())
}
