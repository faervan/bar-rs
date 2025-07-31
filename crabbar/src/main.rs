use clap::Parser as _;
use cli::{CliArgs, handle_cli_commands};

mod cli;
mod config;
mod daemon;
mod logger;
mod message;
mod state;
mod subscription;

fn main() -> anyhow::Result<()> {
    logger::init()?;

    let args = CliArgs::parse();
    log::info!("{args:#?}");

    handle_cli_commands(args)
}
