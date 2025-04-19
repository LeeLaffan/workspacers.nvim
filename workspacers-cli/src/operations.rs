use common::json::{self, Workspace};
use common::setup::config::AppConfig;
use log::info;
use std::env::current_dir;

pub fn add_workspace(cfg: &AppConfig) -> Result<(), String> {
    let mut workspaces = json::read_workspaces(&cfg.json_path)?;
    let new_ws = read_new_workspace(&workspaces)?;
    (println!("Adding Workspace: {new_ws}"));
    workspaces.push(new_ws);
    json::write_workspaces(&cfg.json_path, &workspaces)
}

fn read_new_workspace(workspaces: &Vec<Workspace>) -> Result<Workspace, String> {
    let mut cwd = current_dir()
        .map_err(|_| "Failed to get current directory")?
        .to_string_lossy()
        .to_string();
    if !cwd.ends_with(std::path::MAIN_SEPARATOR) {
        cwd.push(std::path::MAIN_SEPARATOR);
    }

    let path = read_line("Path".to_string(), &cwd)?;
    if workspaces.iter().any(|ws| ws.path == path) {
        return Err(format!("Workspace Path already exists: '{path}'"));
    }

    let name = read_line("Name".to_string(), "")?;
    if workspaces.iter().any(|ws| ws.name == name) {
        return Err(format!("Workspace Name already exists: '{name}'"));
    }

    let sc = read_line("Shortcut".to_string(), "")?;
    if workspaces.iter().any(|ws| ws.shortcut == sc) {
        return Err(format!("Workspace Shortcut already exists: '{sc}'"));
    }

    Ok(Workspace {
        name,
        shortcut: sc,
        path,
    })
}

pub fn delete_workspace(del_ws: &Workspace, cfg: &AppConfig) -> Result<(), String> {
    let workspaces = json::read_workspaces(&cfg.json_path)?;
    info!("Request to delete Workspace: {del_ws}");
    let remaining_ws: Vec<&json::Workspace> = workspaces
        .iter()
        .filter(|ws| del_ws.shortcut.ne(&ws.shortcut))
        .collect();
    if remaining_ws.len() == workspaces.len() {
        return Err(format!("Could not match Workspace to delete: {del_ws}"));
    }

    println!("Are you sure you with to delete: {del_ws}");
    match read_line("confirmation(y/n)".to_string(), "")?.as_str() {
        "y" => json::write_workspaces(&cfg.json_path, &remaining_ws).map(|_| println!("Deleted Workspace: {del_ws}")),
        _ => Err("Cancelled deleting workspace".to_string()),
    }
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

pub fn move_workspace(ws: &Workspace, cfg: &AppConfig, up: bool) -> Result<usize, String> {
    let mut workspaces = json::read_workspaces(&cfg.json_path)?;
    let direction = if up { "increment" } else { "decrement" };
    info!("Request to {} Workspace: {ws}", direction);

    // Find the index of the workspace to move
    let workspace_index = workspaces.iter().position(|w| ws.shortcut.eq(&w.shortcut));

    match workspace_index {
        Some(index) => {
            let len = workspaces.len();
            if len <= 1 {
                return Err(format!("Cannot {} workspace - need at least two workspaces", direction));
            }

            // Calculate new position with wrap-around
            let new_index = if up {
                // Move up (decrement index) with wrap to bottom
                if index == 0 { len - 1 } else { index - 1 }
            } else {
                // Move down (increment index) with wrap to top
                if index >= len - 1 { 0 } else { index + 1 }
            };

            // Perform the move
            let workspace = workspaces.remove(index);
            workspaces.insert(new_index, workspace);
            json::write_workspaces(&cfg.json_path, &workspaces)?;
            Ok(new_index)
        }
        None => Err(format!("Could not match Workspace to {}: {ws}", direction)),
    }
}
