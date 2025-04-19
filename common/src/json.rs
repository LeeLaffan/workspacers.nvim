use serde::Serialize;
use std::fmt;
use std::fs::File;
use std::io::Error;
use std::io::{BufReader, BufWriter};
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

pub fn get_json_path(json_arg: Option<PathBuf>, app_name: &str) -> Result<PathBuf, Error> {
    Ok(match json_arg {
        Some(arg_file) => arg_file,
        None => path::get_data_dir(app_name)?.join(format!("{app_name}.json")),
    })
}

// Retuns an empty Vec when file not found or malformed
pub fn read_workspaces(json_path: &PathBuf) -> Vec<Workspace> {
    let file = match File::open(json_path) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };
    match serde_json::from_reader(BufReader::new(file)) {
        Ok(workspaces) => workspaces,
        Err(_) => Vec::new(),
    }
}

// If json path is not found, it will be created here
pub fn write_workspaces<T>(json_path: &PathBuf, workspaces: &Vec<T>) -> Result<(), String>
where
    T: Serialize,
{
    let file = File::create(json_path).map_err(|err| {
        format!(
            "Failed to create json file: {}, err: {err}",
            json_path.to_string_lossy().to_string()
        )
    })?;
    serde_json::to_writer_pretty(BufWriter::new(file), workspaces).map_err(|err| {
        format!(
            "Failed to write workspaces json: {}, err: {err}",
            json_path.to_string_lossy().to_string()
        )
    })
}
