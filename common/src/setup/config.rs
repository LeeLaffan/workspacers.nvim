use config::{Config, File};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
};
use toml;

use super::path::get_data_dir;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub json_path: String,
}

impl AppConfig {
    pub fn default() -> AppConfig {
        AppConfig {
            json_path: "".to_string(),
        }
    }
}

pub fn get_config(config_args: Option<PathBuf>, app_name: &str) -> Result<(String, AppConfig), Error> {
    let config_file = match config_args {
        Some(cfg_file) => cfg_file,
        None => {
            let cfg_file = get_data_dir(app_name)?.join(format!("{app_name}.toml"));
            if !cfg_file.exists() {
                if let Some(parent) = cfg_file.parent() {
                    std::fs::create_dir_all(parent).map_err(|err| {
                        error!("Failed to create directory: {err}");
                        Error::new(ErrorKind::Other, "Failed to create config directory")
                    })?;
                }

                // Create a default config
                let default_config = AppConfig::default(); // You'll need to implement Default for AppConfig

                // Write the default config to the file
                let toml_string = toml::to_string(&default_config).map_err(|err| {
                    error!("Failed to serialize default config: {err}");
                    Error::new(ErrorKind::Other, "Failed to create default config")
                })?;

                std::fs::write(&cfg_file, toml_string).map_err(|err| {
                    error!("Failed to write default config: {err}");
                    Error::new(ErrorKind::Other, "Failed to write default config file")
                })?;
            }
            cfg_file
        }
    }
    .to_string_lossy()
    .to_string();

    Ok((
        config_file.to_string(),
        Config::builder()
            .add_source(File::with_name(&config_file))
            .build()
            .map_err(|err| {
                error!("{err}");
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Could not build config file: {}", config_file),
                )
            })?
            .try_deserialize()
            .map_err(|err| {
                error!("{err}");
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Could not deserialize config file: {}", config_file),
                )
            })?,
    ))
}
