use clap::Subcommand;
use serde::{Deserialize, Serialize};

pub mod daemon;

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum IpcCommand {
    Open,
    Close,
    Restart,
}
