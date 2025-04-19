use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
};

use log::{LevelFilter, info};
use simplelog::WriteLogger;

use crate::setup::path;

pub fn setup_logger() -> Result<PathBuf, Error> {
    let binary_name = path::get_binary_name()?;
    let log_file = match dirs_next::cache_dir() {
        Some(path) => path.join(format!("{binary_name}.log")),
        None => {
            return Err(Error::new(ErrorKind::NotFound, "Cache directory not found"));
        }
    };

    if WriteLogger::init(
        LevelFilter::Debug,
        simplelog::Config::default(),
        std::fs::File::create(&log_file)?,
    )
    .is_err()
    {
        return Err(Error::new(
            ErrorKind::ResourceBusy,
            "Could not create logger(is another logger already initialised?)",
        ));
    }
    info!("Starting app: {binary_name}");
    Ok(log_file)
}
