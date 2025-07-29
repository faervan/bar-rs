use std::{
    fs,
    io::Write as _,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use ipc::IpcRequest;
use log::{error, info};
use nix::unistd::Pid;

use crate::{daemon, directories};

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
    #[command(display_order = 0)]
    /// Open the `crabbar` daemon
    Open {
        #[arg(short = 'd', long)]
        /// Only start the daemon, don't open any windows
        dry: bool,
        #[arg(short = 'D', long)]
        /// Keep `crabbar` attached to this terminal
        dont_daemonize: bool,
        #[arg(long, default_value = get_log_dir())]
        /// Log file directory. Only applies when the process is daemonized.
        log_dir: PathBuf,
    },
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
            let pid_path = args.run_dir.join("crabbar.pid");

            if fs::exists(&socket_path)? {
                if fs::read_to_string(&pid_path)
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                    .is_some_and(|pid| nix::sys::signal::kill(Pid::from_raw(pid), None).is_ok())
                {
                    return Err(anyhow::anyhow!("`crabbar` is running already!"));
                }
                info!("The previous crabbar instance did not exit gracefully, removing the socket file.");
                if let Err(e) = fs::remove_file(&socket_path) {
                    error!("Could not remove socket file at {socket_path:?}: {e}");
                }
            }

            let socket_path2 = socket_path.clone();
            let pid_path2 = pid_path.clone();
            ctrlc::set_handler(move || {
                daemon::exit_cleanup(&socket_path2, &pid_path2);
                std::process::exit(0);
            })?;

            daemon::run(!dry, !dont_daemonize, &log_dir, socket_path, pid_path)?;
        }
        Command::Ipc(cmd) => {
            let response = ipc::request(cmd, &socket_path)?;
            use ipc::IpcResponse::*;
            match response {
                WindowList(windows) => match windows.is_empty() {
                    true => info!("No windows are open!"),
                    false => info!(
                        "{} windows are open: {}",
                        windows.len(),
                        windows
                            .iter()
                            .map(usize::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                },
                Closing => info!("Closing the crabbar daemon."),
                Error(msg) => error!("{msg}"),
                Window { id, event } => todo!(),
            }
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

fn get_log_dir() -> std::ffi::OsString {
    let home = std::env::var("HOME").unwrap();
    let fallback_dir = from_env_or(format!("{home}/.local/state"), "XDG_STATE_HOME");
    from_env_or(fallback_dir, "CRABBAR_LOG_DIR")
}
