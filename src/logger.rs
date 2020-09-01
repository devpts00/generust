use core::result;
use fern::colors::{Color, ColoredLevelConfig};

pub fn setup() -> result::Result<(), fern::InitError> {

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
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .map_err(|err| fern::InitError::SetLoggerError(err))
}
