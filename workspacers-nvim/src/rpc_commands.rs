use async_trait::async_trait;
use common::{
    formatter,
    json::{self, Workspace},
};
use log::{error, info};
use nvim_rs::{Handler, Neovim, Value, compat::tokio::Compat};
use std::{io::Error, path::PathBuf};

#[derive(Clone)]
pub struct NeovimHandler {
    pub json_dir: PathBuf,
    pub log_file: PathBuf,
}

// Request
const RPC_WS_LIST: &str = "WORKSPACERS.LIST";
const RPC_WS_ADD: &str = "WORKSPACERS.ADD";
const RPC_WS_DELETE: &str = "WORKSPACERS.DELETE";
const RPC_WS_JSON: &str = "WORKSPACERS.JSON";
const RPC_WS_PROMOTE: &str = "WORKSPACERS.PROMOTE";
const RPC_WS_DEMOTE: &str = "WORKSPACERS.DEMOTE";
const RPC_WS_RECORD: &str = "WORKSPACERS.RECORD";
const RPC_WS_REPLACE: &str = "WORKSPACERS.REPLACE";

fn rpc_cmd<T>(command_name: &str, result: Result<T, impl std::fmt::Debug>) -> Result<T, Value> {
    result.map_err(|_| Value::String(format!("Error running {command_name}").into()))
}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<tokio::io::Stdout>;

    // Requests will respond with a Value to lua
    async fn handle_request(
        &self,
        name: String,
        args: Vec<Value>,
        _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        info!("REQUEST: {}, {:?}", name, args);
        let response = handle_req(name, args, &self.json_dir);
        if response.is_ok() {
            info!("RESPONSE: {}", response.to_owned().unwrap());
        } else {
            error!("ERROR: {}", response.as_ref().err().unwrap());
        }
        response
    }
}

fn handle_req(name: String, args: Vec<Value>, json_dir: &PathBuf) -> Result<Value, Value> {
    let ws_arg = args[0].as_str().unwrap();
    let json_path = &json::get_json_file(json_dir, ws_arg);

    if name == RPC_WS_JSON {
        return Ok(Value::String(json_dir.to_string_lossy().into()));
    }
    info!("Received arg[0]: {}", args[0]);

    let workspaces = json::read_workspaces(json_path); // Read the json once at the top level 

    match name.as_str() {
        RPC_WS_LIST => rpc_cmd(RPC_WS_LIST, rpc_ws_list(&workspaces)),
        RPC_WS_RECORD => rpc_cmd(RPC_WS_RECORD, rpc_ws_record(&workspaces, args)),

        RPC_WS_ADD => rpc_cmd(RPC_WS_ADD, rpc_ws_add(workspaces, json_path, args)),
        RPC_WS_DELETE => rpc_cmd(RPC_WS_DELETE, rpc_ws_delete(&workspaces, json_path, args)),

        RPC_WS_PROMOTE => rpc_cmd(RPC_WS_PROMOTE, rpc_ws_promote(&workspaces, json_path, args)),
        RPC_WS_DEMOTE => rpc_cmd(RPC_WS_DEMOTE, rpc_ws_demote(&workspaces, json_path, args)),

        RPC_WS_REPLACE => rpc_cmd(RPC_WS_REPLACE, rpc_ws_replace(workspaces, json_path, args)),
        _ => {
            error!("Unknown request: {}", name);
            Ok(Value::Boolean(false))
        }
    }
}

/// Sends an array of objects in the form:
/// [
///    {
///        "Fmt": " [ Entry1 ] - [ Path1 ] ",
///        "Workspace": {
///           "Name": "Entry1"
///           "Path": "Path1"
///        }
///    },
///    { ... }
/// ],
fn rpc_ws_list(workspaces: &Vec<Workspace>) -> Result<Value, String> {
    let result = formatter::fmt(workspaces)
        .iter()
        .map(|(ws_str, ws)| {
            let mut map = Vec::new();

            let mut workspace_map = Vec::new();
            workspace_map.push((Value::String("Name".into()), Value::String(ws.name.to_string().into())));
            workspace_map.push((Value::String("Path".into()), Value::String(ws.path.to_string().into())));

            map.push((
                Value::String(ws_str.to_string().into()),
                Value::Map(workspace_map.into()),
            ));
            Value::Map(map.into())
        })
        .collect::<Vec<Value>>();

    Ok(Value::Array(result))
}

fn rpc_ws_record(workspaces: &Vec<Workspace>, args: Vec<Value>) -> Result<Value, String> {
    info!("request to pick: {}", args[1]);
    let arg_pick = args[1].as_str().unwrap();
    match formatter::fmt(&workspaces)
        .iter()
        .find(|(ws_str, _)| arg_pick.eq(ws_str))
        .map(|(_, ws)| ws)
    {
        Some(ws_match) => {
            info!("picking: {}", ws_match.name);
            Ok(Value::Map(vec![
                (
                    Value::String("Name".into()),
                    Value::String(ws_match.name.to_string().into()),
                ),
                (
                    Value::String("Path".into()),
                    Value::String(ws_match.path.to_string().into()),
                ),
            ]))
        }
        None => Err("No matching workspace".to_string()),
    }
}

fn rpc_ws_add(mut workspaces: Vec<Workspace>, json_file: &PathBuf, args: Vec<Value>) -> Result<Value, Error> {
    if let Some(ws_arg) = args[1].as_map() {
        let ws = json::Workspace {
            name: convert_ws_add(ws_arg, "name")?,
            path: formatter::unfmt_path(convert_ws_add(ws_arg, "path")?),
        };
        info!("req to add: {ws} to json file: {}", json_file.to_string_lossy());
        workspaces.insert(workspaces.len(), ws);
        match json::write_workspaces(json_file, &workspaces) {
            Ok(()) => Ok(Value::Boolean(true)),
            Err(e) => {
                error!("Could not write workspace: {}", e);
                Err(Error::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not write workspace: {e}"),
                )))
            }
        }
    } else {
        // TODO LL
        Ok(Value::Boolean(false))
    }
}

fn rpc_ws_delete(workspaces: &Vec<Workspace>, json_file: &PathBuf, args: Vec<Value>) -> Result<Value, Error> {
    info!("req to del: {}", args[1]);
    let fmt_vals = formatter::fmt(&workspaces);
    info!("count before del: {}", workspaces.len());
    let ws_fmt_arg = args[1].as_str().unwrap();
    let remaining_ws: Vec<&json::Workspace> = fmt_vals
        .iter()
        .filter(|(ws_str, _)| !ws_fmt_arg.eq(ws_str))
        .map(|(_, ws)| ws)
        .collect();
    info!("count after del: {}", remaining_ws.len());

    match json::write_workspaces(json_file, &remaining_ws) {
        Ok(()) => Ok(Value::Boolean(true)),
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not write workspace",
        )),
    }
}

fn rpc_ws_replace(mut workspaces: Vec<Workspace>, json_file: &PathBuf, args: Vec<Value>) -> Result<Value, Error> {
    let fmt_vals = formatter::fmt(&workspaces);
    let arg_pairs = args[1]
        .as_map()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Invalid arguments"))?;

    // Find the key and new values
    let key = arg_pairs
        .iter()
        .find(|(k, _)| k.as_str().unwrap() == "Key")
        .and_then(|(_, v)| v.as_str())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Invalid Key"))?;

    let new_values = arg_pairs
        .iter()
        .find(|(k, _)| k.as_str().unwrap() == "New")
        .and_then(|(_, v)| v.as_map())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Invalid New values"))?;

    // Find the workspace to replace
    if let Some(idx) = fmt_vals.iter().position(|(ws_str, _)| key.eq(ws_str)) {
        let name_value = new_values
            .iter()
            .find(|(k, _)| k.as_str().unwrap() == "Name")
            .and_then(|(_, v)| v.as_str())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Missing Name"))?;
        let path_value = new_values
            .iter()
            .find(|(k, _)| k.as_str().unwrap() == "Path")
            .and_then(|(_, v)| v.as_str())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Missing Path"))?;

        workspaces[idx] = json::Workspace {
            name: name_value.to_string(),
            path: formatter::unfmt_path(path_value.to_string()),
        };

        // Write updated workspaces
        json::write_workspaces(json_file, &workspaces)
            .map(|_| Value::Boolean(true))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Write error: {e}")).into())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Workspace not found").into())
    }
}

fn rpc_ws_promote(workspaces: &Vec<Workspace>, json_file: &PathBuf, args: Vec<Value>) -> Result<Value, Error> {
    let fmt_vals = formatter::fmt(workspaces);
    let ws_fmt_arg = args[1].as_str().unwrap();
    let mut new_idx = 0;
    let mut new_workspaces: Vec<&json::Workspace> = fmt_vals.iter().map(|(_, ws)| ws).collect();
    if let Some(idx) = fmt_vals.iter().position(|(ws_str, _)| ws_fmt_arg.eq(ws_str)) {
        let target_idx = if idx == 0 { new_workspaces.len() - 1 } else { idx - 1 };
        let ws = new_workspaces.remove(idx);
        new_workspaces.insert(target_idx, ws);
        new_idx = target_idx;
    }
    match json::write_workspaces(json_file, &new_workspaces) {
        Ok(()) => Ok(Value::Integer(new_idx.into())), // Return the new index
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not write workspace",
        )),
    }
}

fn rpc_ws_demote(workspaces: &Vec<Workspace>, json_file: &PathBuf, args: Vec<Value>) -> Result<Value, Error> {
    let fmt_vals = formatter::fmt(workspaces);
    let ws_fmt_arg = args[1].as_str().unwrap();
    let mut new_idx = 0;
    let mut new_workspaces: Vec<&json::Workspace> = fmt_vals.iter().map(|(_, ws)| ws).collect();
    if let Some(idx) = fmt_vals.iter().position(|(ws_str, _)| ws_fmt_arg.eq(ws_str)) {
        let target_idx = if idx == new_workspaces.len() - 1 { 0 } else { idx + 1 };
        let ws = new_workspaces.remove(idx);
        new_workspaces.insert(target_idx, ws);
        new_idx = target_idx;
    }
    match json::write_workspaces(json_file, &new_workspaces) {
        Ok(()) => Ok(Value::Integer(new_idx.into())), // Return the new index
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not write workspace",
        )),
    }
}

pub fn convert_ws_add(obj: &Vec<(Value, Value)>, prop: &str) -> Result<String, Error> {
    if let Some(prop_match) = obj.iter().find(|p| p.0.as_str().unwrap() == prop) {
        info!("match to add: {}", &prop_match.1.to_string());
        Ok(formatter::unfmt_ws_value(&prop_match.1.to_string()))
    } else {
        error!("could not read workspace add data - inner");
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "could not read workspace add data - inner",
        ))
    }
}
