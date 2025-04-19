use std::{io::Write, path::PathBuf};

use clap::{Parser, command};
use common::{
    formatter, json,
    setup::{config, logging},
};

mod operations;
mod picker;

#[derive(Debug)]
pub enum AppError {
    Cancelled(std::io::Error),
    InvalidData(String),
    NotFound,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    }
}

// Use Result<_, String> throughout in order to capture errors to display to user
fn run() -> Result<(), String> {
    let args = CliArgs::parse();
    logging::setup_logger().map_err(|err| format!("Could not setup logger: {err}"))?;
    let cfg = config::get_config().map_err(|err| format!("Could not setup config: {err}"))?;

    if args.add {
        if args.shortcut.is_some() {
            return Err("Cannot combine args: add & shortcut".to_string());
        }
        let ws = operations::add_workspace(&cfg)?;
        return Ok(());
    }
    if args.delete {
        if args.shortcut.is_none() {
            return Err("Cannot delete without shortcut".to_string());
        }
        // let ws_match = return operations::delete_workspace(&workspaces, & cfg, args.shortcut.unwrap());
        todo!();
    }
    if args.shortcut.is_some() {
        todo!()
    }

    match picker::pick_workspace(&cfg)? {
        None => Ok(()), // Don't print when no workspace selected
        Some(ws) => {
            if !args.raw && args.pipe_file.is_some() {
                // println!("Workspace selected! 🚀{ws}");
                println!("{}", ws.path);
                return Ok(write_cd(&args.pipe_file.unwrap(), &ws.path)?);
            }
            return Ok(println!("{}", &ws.path));
        }
    }
}

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct CliArgs {
    /// Add a new Workspace
    #[arg(short = 'a', long, default_value_t = false)]
    add: bool,

    /// Delete [SHORTCUT]
    #[arg(short = 'd', long, default_value_t = false, requires = "shortcut")]
    delete: bool,

    /// Opens the Workspace with editor (configurable)
    #[arg(short = 'r', long, default_value_t = false)]
    open: bool,

    /// Shortcut string (a single positional argument without a dash)
    #[arg(index = 1, value_name = "SHORTCUT")]
    shortcut: Option<String>,

    #[arg(long = "PIPE_FILE", hide = true)]
    pipe_file: Option<std::path::PathBuf>,

    /// Raw prints the result (for piping)
    #[arg(short = 'r', long, default_value_t = false)]
    raw: bool,
}

fn write_cd(pipe_file: &PathBuf, dir: &str) -> Result<(), String> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(pipe_file)
        .map_err(|e| format!("Failed to open command pipe: {}", e))?;

    let mut writer = std::io::BufWriter::new(file);
    writer
        .write_fmt(format_args!("cd {}; ls;", dir))
        .map_err(|e| format!("Failed to write command: {}", e))?;

    writer.flush().map_err(|e| format!("Failed to flush command: {}", e))
}
