use clap::Parser as _;
use cli::{handle_cli_commands, CliArgs};

mod cli;
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
