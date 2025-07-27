use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum IpcRequest {
    #[command(name = "list")]
    /// List all open windows
    ListWindows,
    #[command(name = "window")]
    /// Perform a window action
    WindowCommand {
        #[command(subcommand)]
        cmd: WindowCommand,
        /// Optional window ID
        // TODO!: has to be None if cmd is Open
        id: Option<usize>,
    },
    #[command(skip)]
    CloseAll,
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum WindowCommand {
    /// Open a new window
    Open,
    /// Close a window
    Close,
    /// Reopen a window to apply settings like bar height/width
    Reopen,
}

pub enum IpcResponse {
    WindowOpened { id: usize },
    WindowList(Vec<usize>),
}
