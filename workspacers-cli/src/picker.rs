use common::json::Workspace;
use fzf_wrapped::{Fzf, Layout};
use log::info;

// Returns an Option Some Workspace or None exited safely
pub fn pick_workspace(workspaces: Vec<Workspace>) -> Result<Option<Workspace>, String> {
    let fmt_vals = common::formatter::fmt(&workspaces);
    let vals: Vec<String> = fmt_vals.iter().map(|(ws_str, _)| ws_str.to_string()).collect();

    match run_fzf(vals)? {
        None => return Ok(None), // If user cancelled, do not error
        Some(fzf_output) => {
            let lines = filter_result(fzf_output);
            if lines.is_empty() {
                info!("No lines returned from fzf");
                return Ok(None);
            }

            let ws_match = fmt_vals
                .iter()
                .find(|ws| ws.0 == lines[0])
                .ok_or_else(|| "Could not match fmt back to Workspace".to_string())?;

            match lines.len() {
                1 => Ok(Some(ws_match.1.clone())), // This function expects a new WS for adding so clone to allow
                _ => Err("Unexpected number of args".to_string()),
            }
        }
    }
}

fn run_fzf(values: Vec<String>) -> Result<Option<String>, String> {
    let mut fzf = Fzf::builder()
        .layout(Layout::Reverse)
        .header("Workspace:")
        .build()
        .or_else(|err| Err(format!("fzf - Could not build: {err}")))?;

    match fzf.run() {
        Ok(()) => {
            fzf.add_items(values)
                .map_err(|err| format!("fzf - could add value: {err}"))?;
            match fzf.output() {
                Some(sel) => Ok(Some(sel)),
                None => Err("fzf - could not select value".to_string()),
            }
        }
        Err(e) => Err(format!("fzf - could not run: {e}")),
    }
}

fn filter_result(output: String) -> Vec<String> {
    output
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .rev() // Reverse to get selected first
        .map(String::from)
        .collect()
}
