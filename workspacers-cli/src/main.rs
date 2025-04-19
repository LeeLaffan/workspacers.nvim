use clap::{Parser, command};
use common::{json, setup::logging};

mod add_workspace;
mod picker;

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct CliArgs {
    /// JSON File to be used to store workspaces
    #[arg(long = "json-file", hide = false)]
    json_file: Option<std::path::PathBuf>,

    /// Print the JSON file used
    #[arg(short = 'j', long, default_value_t = false)]
    print_json: bool,

    /// Add a new Workspace
    #[arg(short = 'a', long, default_value_t = false)]
    add: bool,
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
    let json_file = json::get_json_path(args.json_file, "workspacers").unwrap();
    if args.print_json {
        println!("{}", json_file.to_string_lossy().to_string());
        return Ok(());
    }
    let workspaces = json::read_workspaces(&json_file);

    if args.add {
        add_workspace::add(&workspaces, &json_file)?;
        return Ok(());
    }

    match picker::pick_workspace(workspaces)? {
        None => Ok(()), // Don't print when no workspace selected
        Some(ws) => {
            return Ok(println!("{}", &ws.path));
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    Cancelled(std::io::Error),
    InvalidData(String),
    NotFound,
}
