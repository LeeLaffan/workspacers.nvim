use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
};

pub fn get_binary_name() -> Result<String, Error> {
    let binary_path = std::env::current_exe()
        .map_err(|e| Error::new(ErrorKind::NotFound, format!("Failed to get executable path: {}", e)))?;

    let binary_name = binary_path.file_stem().and_then(|name| name.to_str());
    match binary_name {
        Some(name) => Ok(name.to_string()),
        None => Err(Error::new(ErrorKind::NotFound, "Failed to get executable name")),
    }
}

pub fn get_data_dir(app_name: &str) -> Result<PathBuf, Error> {
    match dirs_next::data_local_dir() {
        Some(data_dir) => {
            let app_dir = data_dir.join(app_name);
            if !app_dir.exists() {
                std::fs::create_dir(app_dir)?;
            }
            Ok(data_dir.join(app_name))
        }
        None => {
            return Err(Error::new(ErrorKind::NotFound, "DataLocal directory not found"));
        }
    }
}
