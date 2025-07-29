use fern::colors::ColoredLevelConfig;

pub fn init() -> Result<(), log::SetLoggerError> {
    let colors = ColoredLevelConfig::new()
        .trace(fern::colors::Color::BrightBlue)
        .debug(fern::colors::Color::BrightMagenta)
        .info(fern::colors::Color::Blue)
        .warn(fern::colors::Color::Magenta)
        .error(fern::colors::Color::Red);

    fern::Dispatch::new()
        .format(move |out, msg, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%d/%m/%Y %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                msg
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
}
