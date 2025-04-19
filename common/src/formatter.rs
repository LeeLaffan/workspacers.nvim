use std::env;
use std::path::{Path, PathBuf};

use crate::json::Workspace;

pub fn fmt(workspaces: &Vec<Workspace>) -> Vec<(String, Workspace)> {
    if workspaces.is_empty() {
        return Vec::new();
    }
    let fmt_vals = workspaces
        .iter()
        .map(|ws| {
            (
                Workspace {
                    name: ws.name.to_string(),
                    path: fmt_path(ws.path.to_string()),
                },
                ws.clone(),
            )
        })
        .collect::<Vec<(Workspace, Workspace)>>();

    let longest_name = fmt_vals.iter().map(|(fmt, _)| fmt.name.len()).max().unwrap_or(0);
    let longest_path = fmt_vals.iter().map(|(fmt, _)| fmt.path.len()).max().unwrap_or(0);

    fmt_vals
        .iter()
        .map(|(fmt_ws, orig_ws)| {
            let path = Path::new(&orig_ws.path); // Use original path for directory check
            let icon = if path.is_dir() { "📁" } else { "📄" };

            let formatted_string = format!(
                "[ {} ] - [ {} ] - [ {} ]",
                icon,
                pad_right(fmt_ws.name.to_string(), longest_name),
                pad_right(fmt_ws.path.to_string(), longest_path)
            );

            (formatted_string, orig_ws.to_owned())
        })
        .collect()
}

fn pad_right(s: String, width: usize) -> String {
    format!("{:<width$}", s, width = width)
}

/// Formats a raw string by removing surrounding whitespace and quotes.
pub fn unfmt_ws_value(raw: &str) -> String {
    let trimmed = raw.trim();

    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        // Remove the first and last character (the quotes)
        let without_quotes = &trimmed[1..trimmed.len() - 1];
        // Trim again in case there was whitespace inside the quotes
        without_quotes.trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Formats a path string to ensure it has a trailing separator and expands the tilde character.
pub fn unfmt_path(path: String) -> String {
    // Step 0: Trim any surrounding quotes
    let path = path.trim_matches('"').trim_matches('\'');

    // Step 1: Expand tilde if it exists at the beginning of the path
    let expanded_path = if path.starts_with("~") {
        match home_dir() {
            Some(mut home) => {
                // Remove the tilde and any separator that might follow it
                let remainder = path.strip_prefix("~").unwrap();
                let remainder = remainder
                    .strip_prefix('/')
                    .or_else(|| remainder.strip_prefix('\\'))
                    .unwrap_or(remainder);

                // Append the remainder of the path to the home directory
                home.push(remainder);
                home
            }
            None => PathBuf::from(path), // If we can't determine the home dir, return original
        }
    } else {
        PathBuf::from(path)
    };

    expanded_path.to_string_lossy().to_string()
}

// Formats a path replacing home dir with ~
pub fn fmt_path(path: String) -> String {
    if let Some(home) = home_dir() {
        let home_str = home.to_string_lossy().to_string();

        if path.starts_with(&home_str) {
            return format!("~{}", &path[home_str.len()..]);
        }
    }

    path
}

pub fn home_dir() -> Option<PathBuf> {
    // Try to get home directory from environment variables
    if cfg!(windows) {
        // On Windows, use %USERPROFILE%
        env::var_os("USERPROFILE").map(PathBuf::from)
    } else {
        // On Unix, use $HOME
        env::var_os("HOME").map(PathBuf::from)
    }
}
