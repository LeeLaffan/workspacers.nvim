use log::{error, info};
use nvim_rs::Value;
use std::{env, io::Error, path::PathBuf};

pub fn convert_ws_add(obj: &Vec<(Value, Value)>, prop: &str) -> Result<String, Error> {
    if let Some(prop_match) = obj.iter().find(|p| p.0.as_str().unwrap() == prop) {
        info!("match to add: {}", &prop_match.1.to_string());
        Ok(fmt_ws_value(&prop_match.1.to_string()))
    } else {
        error!("could not read workspace add data - inner");
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "could not read workspace add data - inner",
        ))
    }
}

/// Formats a raw string by removing surrounding whitespace and quotes.
pub fn fmt_ws_value(raw: &str) -> String {
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
pub fn fmt_path(path: String) -> String {
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
