use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum IpcCommand {
    Open,
    Close,
    Restart,
}
