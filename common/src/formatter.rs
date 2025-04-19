use crate::json::Workspace;

pub fn fmt(workspaces: &Vec<Workspace>) -> Vec<(String, Workspace)> {
    if workspaces.is_empty() {
        return Vec::new();
    }

    let longest_name = get_longest(workspaces, |w| &w.name);
    let longest_path = get_longest(workspaces, |w| &w.path);

    workspaces
        .iter()
        .map(|ws| (fmt_ws_pad(ws, (longest_name, longest_path)), ws.clone()))
        .collect()
}

fn fmt_ws_pad(ws: &Workspace, padding: (usize, usize)) -> String {
    format!(
        "[ {} ] - [ {} ]",
        pad_right(ws.name.to_string(), padding.0),
        pad_right(ws.path.to_string(), padding.1)
    )
}

fn get_longest<T>(workspaces: &Vec<Workspace>, prop: T) -> usize
where
    T: Fn(&Workspace) -> &String,
{
    workspaces.iter().map(prop).map(|s| s.len()).max().unwrap_or(0)
}

fn pad_right(s: String, width: usize) -> String {
    format!("{:<width$}", s, width = width)
}
