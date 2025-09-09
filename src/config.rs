use std::fs::{self, File};
// use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub source_path: String,
    pub dest_path: String,
}

fn initialize_config(path: &std::path::Path) {
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
    std::fs::write(path, toml_config).unwrap();
    println!("Configuration file created at {}", path.display());
}

pub fn read_config() -> Result<Config, std::io::Error> {
    // if let Some(path) = config_path.as_deref() {
    //     println!("Configuration file read at {}", path.display());
    //     let content = fs::read_to_string(&path).map_err(|err| {
    //         eprintln!("Failed to read {:?}: {}", path, err);
    //         err
    //     })?;
    //     toml::from_str(&content).map_err(|err| {
    //         eprintln!("Failed to parse config {:?}: {}", path, err);
    //         std::io::Error::new(std::io::ErrorKind::InvalidData, err)
    //     })
    // } else {
    let path = dirs::home_dir()
        .unwrap()
        .join(".config/rustbackup/config.toml");

    match File::open(&path) {
        Ok(_) => {
            // Config file exists, read and parse it
            println!("Configuration file read at {}", path.display());
            let content = fs::read_to_string(&path).map_err(|err| {
                eprintln!("Failed to read {:?}: {}", path, err);
                err
            })?;
            toml::from_str(&content).map_err(|err| {
                eprintln!("Failed to parse config {:?}: {}", path, err);
                std::io::Error::new(std::io::ErrorKind::InvalidData, err)
            })
        }
        Err(_) => {
            // Config file does not exist, create directories and initialize config
            if let Some(parent) = path.parent() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("Cannot create {}: {}", parent.display(), err);
                    return Err(err);
                }
            }
            initialize_config(&path);
            // After initializing, read and parse the config
            let content = fs::read_to_string(&path).map_err(|err| {
                eprintln!("Failed to read {:?}: {}", path, err);
                err
            })?;
            toml::from_str(&content).map_err(|err| {
                eprintln!("Failed to parse config {:?}: {}", path, err);
                std::io::Error::new(std::io::ErrorKind::InvalidData, err)
            })
        }
    }
    // }
}
