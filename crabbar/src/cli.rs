use std::{
    fs,
    io::Write as _,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use ipc::IpcRequest;
use log::{error, info};

use crate::{
    daemon::{create_instance, daemonize},
    directories,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value = get_runtime_dir())]
    /// Runtime directory to be used for IPC socket communication
    run_dir: PathBuf,
    #[arg(short, long, default_value = directories::config().unwrap())]
    /// Path of the main configuration file
    config_path: PathBuf,
    #[arg(long, default_value = directories::theme_dir().unwrap())]
    /// Directory of theme configurations
    theme_dir: PathBuf,
    #[arg(long, default_value = directories::style_dir().unwrap())]
    /// Directory of style configurations
    style_dir: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Open the `crabbar` daemon
    Open {
        #[arg(short = 'd', long)]
        /// Only start the daemon, don't open any windows
        dry: bool,
        #[arg(short = 'D', long)]
        /// Keep `crabbar` attached to this terminal
        dont_daemonize: bool,
        #[arg(long, default_value = from_env_or("/var/log/crabbar", "CRABBAR_LOG_DIR"))]
        /// Log file directory. Only applies when the process is daemonized.
        log_dir: PathBuf,
    },
    /// Close `crabbar` (with all windows)
    Close,
    #[command(flatten)]
    Ipc(IpcRequest),
}

pub fn handle_cli_commands(args: CliArgs) -> anyhow::Result<()> {
    let socket_path = args.run_dir.join("crabbar.sock");

    match args.command {
        Command::Open {
            dry,
            dont_daemonize,
            log_dir,
        } => {
            std::fs::create_dir_all(&args.run_dir)?;
            let mut id = 0;
            if let Some(last_id) = last_instance_id(&args.run_dir) {
                id = last_id + 1;
                return Err(anyhow::Error::msg("`crabbar` is running already!"));
            }

            let path2 = socket_path.clone();

            ctrlc::set_handler(move || {
                if let Err(e) = fs::remove_file(&path2) {
                    error!("Could not remove socket file at {path2:?}: {e}");
                }
                std::process::exit(0);
            })?;

            if !dont_daemonize {
                daemonize(id, &log_dir, &args.run_dir)?;
            }

            create_instance(&socket_path)?;
        }
        Command::Close => {
            send_ipc_command(IpcRequest::CloseAll, &socket_path)?;
        }
        Command::Ipc(cmd) => {
            send_ipc_command(IpcRequest::CloseAll, &socket_path)?;
            // TODO! print response
        }
    }

    Ok(())
}

fn from_env_or<S: AsRef<std::ffi::OsStr>, T: Into<std::ffi::OsString>>(
    default: T,
    key: S,
) -> std::ffi::OsString {
    std::env::var(key)
        .map(Into::into)
        .unwrap_or_else(|_| default.into())
}

fn get_runtime_dir() -> std::ffi::OsString {
    let mut fallback_dir = from_env_or("/tmp", "XDG_RUNTIME_DIR");
    fallback_dir.push("/crabbar");
    from_env_or(fallback_dir, "CRABBAR_RUN_DIR")
}

fn send_ipc_command(cmd: IpcRequest, socket_path: &Path) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.write_all(ron::to_string(&cmd)?.as_bytes())?;

    Ok(())
}
