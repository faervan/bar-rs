use clap::Parser as _;
use cli::{handle_cli_commands, CliArgs};

mod cli;
mod config;
mod daemon;
mod directories;
mod logger;
mod message;
mod state;
mod subscription;
mod window;

fn main() -> anyhow::Result<()> {
    logger::init()?;

    let args = CliArgs::parse();
    log::info!("{args:#?}");

    handle_cli_commands(args)
}
