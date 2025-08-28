use core::{
    config::{style::ContainerStyle, theme::Theme, ConfigOptions, GlobalConfig},
    daemon, directories,
    ipc::{self, IpcRequest, IpcResponse, WindowResponse},
};
use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use log::{error, info};
use nix::unistd::Pid;
use toml_example::TomlExample as _;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value = directories::runtime_dir())]
    /// Runtime directory to be used for IPC socket communication
    run_dir: PathBuf,
    #[arg(short, long, default_value = directories::config_dir())]
    /// Path of the main configuration directory
    config_dir: PathBuf,
    #[arg(long, default_value = directories::log_file())]
    /// Path of the logfile.
    pub log_file: PathBuf,
    #[arg(long, global = true)]
    /// Set the logging level to debug
    pub debug: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(display_order = 0)]
    /// Open the `crabbar` daemon
    Open {
        #[arg(short = 'D', long)]
        /// Keep `crabbar` attached to this terminal
        dont_daemonize: bool,
        /// Open windows using the given configuration presets
        windows: Vec<String>,
    },
    /// Print the default global configuration
    DefaultGlobalConfig,
    /// Print the default window configuration
    DefaultConfig,
    /// Print the default style
    DefaultStyle,
    /// Print the default theme
    DefaultTheme,
    #[command(flatten)]
    Ipc(IpcRequest),
}

pub fn handle_cli_commands(args: CliArgs) -> anyhow::Result<()> {
    let socket_path = args.run_dir.join("crabbar.sock");

    match args.command {
        Command::Open {
            dont_daemonize,
            windows,
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
                info!(
                    "The previous crabbar instance did not exit gracefully, removing the socket file."
                );
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

            daemon::run(
                windows,
                !dont_daemonize,
                socket_path,
                pid_path,
                args.config_dir,
            )?;
        }
        Command::DefaultGlobalConfig => println!("{}", GlobalConfig::toml_example()),
        Command::DefaultConfig => println!("{}", ConfigOptions::toml_example()),
        Command::DefaultStyle => println!("{}", ContainerStyle::toml_example()),
        Command::DefaultTheme => println!("{}", Theme::toml_example()),
        Command::Ipc(cmd) => {
            let response = ipc::request(cmd, &socket_path)?;
            match response {
                IpcResponse::WindowList(windows) => match windows.is_empty() {
                    true => info!("No windows are open!"),
                    false => {
                        info!("{} windows are open:", windows.len(),);
                        let mut windows: Vec<_> = windows.into_iter().collect();
                        windows.sort_by_key(|&(id, _)| id);
                        for (id, window) in windows {
                            println!("{id}:\t{window:#?}")
                        }
                    }
                },
                IpcResponse::ModuleList(mut modules) => match modules.is_empty() {
                    true => info!("No modules are available!"),
                    false => {
                        info!("{} modules available:", modules.len(),);
                        modules.sort();
                        println!("\t{}", modules.join(", "));
                    }
                },
                IpcResponse::ThemeList(themes) => match themes.is_empty() {
                    true => info!("No themes are available!"),
                    false => {
                        info!("{} themes are available:", themes.len(),);
                        let mut themes: Vec<_> = themes.into_iter().collect();
                        themes.sort_by_key(|(id, _)| id.clone());
                        for (name, theme) in themes {
                            println!("{name}:\t{theme:#?}")
                        }
                    }
                },
                IpcResponse::StyleList(styles) => match styles.is_empty() {
                    true => info!("No styles are available!"),
                    false => {
                        info!("{} styles are available:", styles.len(),);
                        let mut styles: Vec<_> = styles.into_iter().collect();
                        styles.sort_by_key(|(id, _)| id.clone());
                        for (name, style) in styles {
                            println!("{name}:\t{style:#?}")
                        }
                    }
                },
                IpcResponse::Closing => info!("Closing the crabbar daemon."),
                IpcResponse::Error(msg) => error!("{msg}"),
                IpcResponse::Window { id, event } => match event {
                    WindowResponse::Opened => info!("Opened new window with id {id:?}"),
                    WindowResponse::Closed => info!("Closed window with id {id:?}"),
                    WindowResponse::Reopened => info!("Reopened window with id {id:?}"),
                    WindowResponse::Config(cfg) => {
                        info!("Configuration of window with id {id:?}:\n{cfg:#?}")
                    }
                    WindowResponse::Theme(theme) => {
                        info!("Theme of window with id {id:?}:\n{theme:#?}")
                    }
                    WindowResponse::Style(style) => {
                        info!("Style of window with id {id:?}:\n{style:#?}")
                    }
                    WindowResponse::ConfigApplied => info!("The configuration has been updated"),
                    WindowResponse::ThemeApplied => info!("The theme has been updated"),
                    WindowResponse::StyleApplied => info!("The style has been updated"),
                },
            }
        }
    }

    Ok(())
}

impl CliArgs {
    pub fn command_is_open(&self) -> bool {
        matches!(self.command, Command::Open { .. })
    }
}
