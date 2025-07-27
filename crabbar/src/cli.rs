use std::{
    fs,
    io::Write as _,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use ipc::IpcCommand;
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
    /// Open `crabbar`
    Open {
        #[arg(short = 'S', long)]
        /// Open an additional instance of `crabbar`
        seperate: bool,
        #[arg(short = 'D', long)]
        /// Do not daemonize the newly opened instance
        dont_daemonize: bool,
        #[arg(long, default_value = from_env_or("/var/log/crabbar", "CRABBAR_LOG_DIR"))]
        /// Log file directory. Only applies when the process is daemonized.
        log_dir: PathBuf,
    },
    /// Close a running instance of `crabbar`
    Close {
        #[arg(short, long)]
        /// The instance ID
        instance: Option<usize>,
    },
    /// Close all running instances of `crabbar`
    CloseAll,
    /// Restart an instance of `crabbar`
    Restart {
        #[arg(short, long)]
        /// The instance ID
        instance: Option<usize>,
    },
    /// Send an IPC command
    Send {
        #[command(subcommand)]
        cmd: IpcCommand,
        #[arg(short, long)]
        /// The instance ID
        instance: Option<usize>,
    },
    /// List all running instances
    Instances,
}

pub fn handle_cli_commands(args: CliArgs) -> anyhow::Result<()> {
    match args.command {
        Command::Close { .. } | Command::Restart { .. } | Command::Send { .. } => {
            if list_instances(&args.run_dir).is_none() {
                return Err(anyhow::Error::msg("No crabbar instances are running."));
            }
        }
        _ => {}
    }

    match args.command {
        Command::Open {
            seperate,
            dont_daemonize,
            log_dir,
        } => {
            std::fs::create_dir_all(&args.run_dir)?;
            let mut id = 0;
            if let Some(last_id) = last_instance_id(&args.run_dir) {
                id = last_id + 1;
                if !seperate {
                    return Err(anyhow::Error::msg(
                        "There is an instance running already! \
                                Try --seperate to create an additional one.",
                    ));
                }
            }

            let path = args.run_dir.join(format!("crabbar{id}.sock"));
            let path2 = path.clone();

            ctrlc::set_handler(move || {
                if let Err(e) = fs::remove_file(&path2) {
                    error!("Could not remove socket file at {path2:?}: {e}");
                }
                std::process::exit(0);
            })?;

            if !dont_daemonize {
                daemonize(id, &log_dir, &args.run_dir)?;
            }

            create_instance(&path)?;
        }
        Command::Close { instance } => {
            send_ipc_command(IpcCommand::Close, instance, &args.run_dir)?;
        }
        Command::CloseAll => {
            for instance in collect_sockets(fs::read_dir(&args.run_dir)?) {
                let mut stream = UnixStream::connect(args.run_dir.join(instance))?;
                stream.write_all(b"close")?;
            }
        }
        Command::Restart { instance } => {
            send_ipc_command(IpcCommand::Restart, instance, &args.run_dir)?;
        }
        Command::Send { cmd, instance } => {
            send_ipc_command(cmd, instance, &args.run_dir)?;
        }
        Command::Instances => match list_instances(&args.run_dir) {
            Some(instances) => {
                info!("{} instance(s) running:", instances.len());
                for instance in instances {
                    info!("\t{instance}");
                }
            }
            None => info!("No instances are running"),
        },
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

fn list_instances(path: &Path) -> Option<Vec<String>> {
    if let Ok(dir) = fs::read_dir(path) {
        let entries = collect_sockets(dir);
        if !entries.is_empty() {
            return Some(entries);
        }
    }
    None
}

/// Filter all `crabbar` socket files from the directory
fn collect_sockets(entries: fs::ReadDir) -> Vec<String> {
    entries
        .flat_map(|r| r.ok().and_then(|e| e.file_name().into_string().ok()))
        .filter(|entry| entry.starts_with("crabbar") && entry.ends_with(".sock"))
        .collect::<Vec<String>>()
}

fn last_instance_id(path: &Path) -> Option<usize> {
    list_instances(path).and_then(|instances| {
        instances
            .into_iter()
            .filter_map(|i| {
                i.strip_prefix("crabbar")?
                    .strip_suffix(".sock")?
                    .parse()
                    .ok()
            })
            .max()
    })
}

fn get_runtime_dir() -> std::ffi::OsString {
    let mut fallback_dir = from_env_or("/tmp", "XDG_RUNTIME_DIR");
    fallback_dir.push("/crabbar");
    from_env_or(fallback_dir, "CRABBAR_RUN_DIR")
}

fn send_ipc_command(
    cmd: IpcCommand,
    instance: Option<usize>,
    run_dir: &Path,
) -> anyhow::Result<()> {
    let instance = instance.unwrap_or_else(|| last_instance_id(run_dir).unwrap_or(0));
    let mut stream = UnixStream::connect(run_dir.join(format!("crabbar{instance}.sock")))?;
    stream.write_all(ron::to_string(&cmd)?.as_bytes())?;

    Ok(())
}
