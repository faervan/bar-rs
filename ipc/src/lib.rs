use core::window::{Window, WindowCommand, WindowOpenOptions};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write as _},
    os::unix::net::UnixStream,
    path::Path,
};

use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum IpcRequest {
    #[command(name = "list")]
    /// List all open windows
    ListWindows,
    /// Perform a window action
    Window {
        #[command(subcommand)]
        cmd: WindowRequest,
        #[arg(long, global = true)]
        /// Optional ID of the window. Will fallback to the most recently opened if not specified.
        id: Option<usize>,
    },
    #[command(display_order = 1)]
    /// Close `crabbar` (with all windows)
    Close,
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum WindowRequest {
    /// Open a new window
    Open(WindowOpenOptions),
    /// Close a window
    Close {
        #[arg(short = 'A', long)]
        /// Close all open windows
        all: bool,
    },
    /// Reopen a window to apply settings like bar height/width
    Reopen {
        #[arg(short = 'A', long)]
        /// Reopen all open windows
        all: bool,
    },
    #[command(flatten)]
    Command(WindowCommand),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IpcResponse {
    WindowList(HashMap<usize, Window>),
    Window {
        id: Vec<usize>,
        event: WindowResponse,
    },
    Closing,
    Error(String),
}

impl IpcResponse {
    pub fn error<S: ToString>(msg: S) -> Self {
        Self::Error(msg.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WindowResponse {
    Opened,
    Closed,
    Reopened,
}

pub fn request(request: IpcRequest, socket_path: &Path) -> anyhow::Result<IpcResponse> {
    if !fs::exists(socket_path)? {
        return Err(anyhow::anyhow!(
            "The crabbar daemon is not running or \
                    the wrong runtime directory is used."
        ));
    }
    let mut stream = UnixStream::connect(socket_path)?;

    let write_buf = ron::to_string(&request)?;
    let buf_len = write_buf.len() as u32;
    stream.write_all(&buf_len.to_ne_bytes())?;
    stream.write_all(write_buf.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(ron::from_str(&response)?)
}
