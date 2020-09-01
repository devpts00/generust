use core::result;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use std::cmp::min;

pub fn setup(verbose: u8) -> result::Result<(), fern::InitError> {

    let levels = vec![
        LevelFilter::Off,
        LevelFilter::Error,
        LevelFilter::Warn,
        LevelFilter::Info,
        LevelFilter::Debug,
        LevelFilter::Trace
    ];

    let level = levels[min(verbose as usize, levels.len() - 1usize)];

    let colors = ColoredLevelConfig::new()
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, msg, rec| {
            out.finish(format_args!(
                "{} - {} - {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                rec.target(),
                colors.color(rec.level()),
                msg
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
        .map_err(|err| fern::InitError::SetLoggerError(err))
}
