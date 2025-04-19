use clap::{Parser, command};
use common::json;
use common::setup::logging::setup_logger;
use log::{error, info};
use nvim_rs::create::tokio as create;
use std::path::PathBuf;
use std::{
    io::{Error, ErrorKind},
    panic,
};
use tokio::runtime;

mod rpc_commands;
mod utils;

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct NvimArgs {
    #[arg(long = "json-file")]
    json_file: Option<PathBuf>,

    #[arg(long = "config-file")]
    config_file: Option<PathBuf>,
}

fn main() -> Result<(), Error> {
    // Catch panics to display in lua
    panic::set_hook(Box::new(move |panic| {
        error!("----- Panic -----");
        error!("{}", panic);
    }));

    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build runtime")
        .block_on(run_workspacers(NvimArgs::parse()))
}

async fn run_workspacers(args: NvimArgs) -> Result<(), Error> {
    let log_file = setup_logger()?;
    let json_file = json::get_json_path(args.json_file, "workspacers")?;
    info!("Using json file: {}", json_file.to_string_lossy());
    let (nvim, io_handler) = create::new_parent(rpc_commands::NeovimHandler { log_file, json_file }).await;
    match io_handler.await {
        Ok(_) => {
            info!("App Completed. Closing");
            Ok(())
        }
        Err(err) => {
            match nvim.err_writeln(&format!("Error: '{}'", err)).await {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to write error to Neovim: '{}'", e),
            };
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("Failed to write error to Neovim: '{}'", err),
            ))
        }
    }
}
