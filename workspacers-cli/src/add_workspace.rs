use common::formatter;
use common::json::{self, Workspace};
use std::env::current_dir;
use std::path::PathBuf;

pub fn add(workspaces: &Vec<Workspace>, json_path: &PathBuf) -> Result<(), String> {
    let new_ws = read_new_workspace(&workspaces)?;
    println!("Adding Workspace: {new_ws}");
    let mut new_workspaces = workspaces.clone();
    new_workspaces.insert(workspaces.len(), new_ws);
    json::write_workspaces(json_path, &new_workspaces)
}

fn read_new_workspace(workspaces: &Vec<Workspace>) -> Result<Workspace, String> {
    let mut cwd = current_dir()
        .map_err(|_| "Failed to get current directory")?
        .to_string_lossy()
        .to_string();
    if !cwd.ends_with(std::path::MAIN_SEPARATOR) {
        cwd.push(std::path::MAIN_SEPARATOR);
    }

    let name = read_line("Name".to_string(), "")?;
    if workspaces.iter().any(|ws| ws.name == name) {
        return Err(format!("Workspace Name already exists: '{name}'"));
    }

    let path = read_line("Path".to_string(), &cwd)?;
    if workspaces.iter().any(|ws| ws.path == path) {
        return Err(format!("Workspace Path already exists: '{path}'"));
    }

    Ok(Workspace {
        name,
        path: formatter::unfmt_path(path),
    })
}

fn read_line(prop: String, initial: &str) -> Result<String, String> {
    let user_value = rustyline::DefaultEditor::new()
        .unwrap()
        .readline_with_initial(&format!("Enter {prop}: "), (initial, ""))
        .map_err(|_| "Operation Cancelled".to_string())?;

    match user_value.is_empty() {
        true => Err(format!("{prop} cannot be empty")),
        false => Ok(user_value),
    }
}
