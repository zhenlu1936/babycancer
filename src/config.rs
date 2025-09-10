use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub source_path: String,
    pub dest_path: String,
}

fn initialize_config(file: &mut File) {
    let config = Config {
        source_path: dirs::home_dir()
            .unwrap()
            .join(".config/rustbackup/source")
            .to_string_lossy()
            .to_string(),
        dest_path: dirs::home_dir()
            .unwrap()
            .join(".config/rustbackup/dest")
            .to_string_lossy()
            .to_string(),
    };

    let toml_config = toml::to_string(&config).unwrap();
    file.write_all(toml_config.as_bytes()).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
}

pub fn check_config_file(config_path: &Option<PathBuf>) -> Result<File, std::io::Error> {
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

pub fn read_config(config_file: &mut File) -> Result<Config, std::io::Error> {
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

pub fn update_config_file(config_file: &mut File, config: &Config) {
    let toml_config = toml::to_string(config).unwrap();
    config_file.set_len(0).unwrap();
    config_file.seek(std::io::SeekFrom::Start(0)).unwrap();
    config_file.write_all(toml_config.as_bytes()).unwrap();
}
