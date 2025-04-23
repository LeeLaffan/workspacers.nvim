use serde::Serialize;
use std::fmt::{self, format};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use serde::Deserialize;

use crate::setup::path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Workspace {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Path")]
    pub path: String,
}

impl fmt::Display for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n\n\t[ {} ] - [ {} ]\n", self.name, self.path)
    }
}

const APP_NAME: &str = "workspacers";

pub fn get_json_dir(json_arg: Option<PathBuf>) -> Result<PathBuf, Error> {
    Ok(json_arg.unwrap_or(path::get_data_dir(APP_NAME)?))
}

pub fn get_json_file(json_dir: &PathBuf, ws_name: &str) -> PathBuf {
    json_dir.join(format!("{ws_name}.json"))
}

// Retuns an empty Vec when file not found or malformed
pub fn read_workspaces(json_file: &PathBuf) -> Vec<Workspace> {
    let file = match File::open(&json_file) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };

    match serde_json::from_reader(BufReader::new(file)) {
        Ok(workspaces) => workspaces,
        Err(_) => Vec::new(),
    }
}

// If json path is not found, it will be created here
pub fn write_workspaces<T>(json_file: &PathBuf, workspaces: &Vec<T>) -> Result<(), Error>
where
    T: Serialize,
{
    let file = File::create(json_file)?;
    serde_json::to_writer_pretty(BufWriter::new(file), workspaces)
        .map_err(|err| Error::from(Error::new(ErrorKind::Other, err)))?;
    Ok(())
}
