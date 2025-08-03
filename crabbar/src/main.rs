use clap::Parser as _;
use cli::{handle_cli_commands, CliArgs};

mod cli;
mod logger;

fn main() -> anyhow::Result<()> {
    logger::init()?;

    let args = CliArgs::parse();
    log::info!("{args:#?}");

    handle_cli_commands(args)
}
