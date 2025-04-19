use common::{
    json::{self, Workspace},
    setup::config::AppConfig,
};
use fzf_wrapped::{Fzf, Layout};
use log::info;

use crate::operations;

fn get_expect_arg() -> String {
    format!("--expect={},{},{},{}", ADD_KEY, DEL_KEY, INC_KEY, DEC_KEY)
}

const ADD_KEY: &str = "ctrl-a";
const DEL_KEY: &str = "ctrl-x";
const INC_KEY: &str = "ctrl-u";
const DEC_KEY: &str = "ctrl-d";

// Returns an Option Some Workspace or None exited safely
pub fn pick_workspace(cfg: &AppConfig) -> Result<Option<Workspace>, String> {
    let workspaces = json::read_workspaces(&cfg.json_path)?;
    if workspaces.is_empty() {
        return Err("No workspaces found. Add one with the -a option.".to_string());
    };

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
                2 => {
                    let key = &lines[1];
                    info!("Key press from fzf: {key}");
                    match key.as_str() {
                        ADD_KEY => match operations::add_workspace(cfg) {
                            Ok(_) => Ok(pick_workspace(&cfg)?),
                            Err(e) => Err(e),
                        },
                        DEL_KEY => match operations::delete_workspace(&ws_match.1, cfg) {
                            Ok(_) => Ok(pick_workspace(&cfg)?),
                            Err(e) => Err(e),
                        },
                        INC_KEY => match operations::move_workspace(&ws_match.1, cfg, true) {
                            Ok(_) => Ok(pick_workspace(&cfg)?),
                            Err(e) => Err(e),
                        },
                        DEC_KEY => match operations::move_workspace(&ws_match.1, cfg, false) {
                            Ok(_) => Ok(pick_workspace(&cfg)?),
                            Err(e) => Err(e),
                        },
                        key => Err(format!("Unrecognised keypress expected from fzf: {key}")),
                    }
                }
                _ => Err("Unexpected number of args".to_string()),
            }
        }
    }
}

// Expect back either "", "<selected>", or "<expect keybind>\n<selected>"
fn run_fzf(values: Vec<String>) -> Result<Option<String>, String> {
    let mut fzf = Fzf::builder()
        .layout(Layout::Reverse)
        .header("Workspace:")
        .custom_args(vec![
            format!("--height={}", values.len() + 4), // TODO Check this when adding configuration
            get_expect_arg(),
        ])
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
