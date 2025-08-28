use clap::Parser as _;
use cli::{handle_cli_commands, CliArgs};

mod cli;
mod logger;

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    log::info!("{args:#?}");

    logger::init(&args.log_file, args.debug, args.command_is_open())?;

    handle_cli_commands(args)
}
