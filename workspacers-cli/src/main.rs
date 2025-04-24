use clap::{Parser, command};
use common::{json, setup::logging};

mod add_workspace;
mod picker;

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct CliArgs {
    /// JSON File to be used to store workspaces
    #[arg(long = "json-file", hide = false)]
    json_dir: Option<std::path::PathBuf>,

    /// Print the JSON file used
    #[arg(short = 'j', long, default_value_t = false)]
    print_json: bool,

    /// Add a new Workspace
    #[arg(short = 'a', long, default_value_t = false)]
    add: bool,

    /// Name of list/json file to target
    #[arg(short = 'n', long, default_value = "workspacers")]
    name: String,
}

// Use Result<_, String> throughout in order to capture errors to display to user
fn main() -> Result<(), String> {
    let args = CliArgs::parse();
    logging::setup_logger().map_err(|err| format!("Could not setup logger: {err}"))?;
    let json_dir = json::get_json_dir(args.json_dir).unwrap();
    let json_file = json_dir.join(format!("{}.json", args.name));

    if args.print_json {
        println!("{}", json_file.to_string_lossy().to_string());
        return Ok(());
    }
    let workspaces = json::read_workspaces(&json_file);

    if args.add {
        add_workspace::add(&workspaces, &json_file)?;
        return Ok(());
    }

    if workspaces.is_empty() {
        return Err("No workspaces found. Add one with the -a option.".to_string());
    };

    match picker::pick_workspace(workspaces)? {
        None => Ok(()), // Don't print when no workspace selected
        Some(ws) => {
            return Ok(println!("{}", &ws.path));
        }
    }
}
