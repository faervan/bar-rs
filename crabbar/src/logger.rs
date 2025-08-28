use std::path::Path;

use fern::colors::ColoredLevelConfig;

pub fn init(log_path: &Path, debug: bool, command_is_open: bool) -> anyhow::Result<()> {
    let colors = ColoredLevelConfig::new()
        .trace(fern::colors::Color::BrightBlue)
        .debug(fern::colors::Color::BrightMagenta)
        .info(fern::colors::Color::Blue)
        .warn(fern::colors::Color::Magenta)
        .error(fern::colors::Color::Red);

    let level = match debug {
        true => log::LevelFilter::Debug,
        false => log::LevelFilter::Info,
    };

    let dispatch = fern::Dispatch::new()
        .format(move |out, msg, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%d/%m/%Y %H:%M:%S:%.3f"),
                colors.color(record.level()),
                record.target(),
                msg
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("crabbar", level)
        .level_for("crabbar_core", level)
        .chain(std::io::stdout());

    if command_is_open {
        dispatch.chain(fern::log_file(log_path)?).apply()?
    } else {
        dispatch.apply()?
    }

    Ok(())
}
