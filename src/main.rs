mod generust;
mod options;
mod logger;

use std::io::{BufWriter, Write, Error};
use options::Options;
use structopt::StructOpt;

fn quit_err<T>(err: Error) -> T {
    log::error!("error: {}", err);
    std::process::exit(err.raw_os_error().unwrap_or_else(|| 1));
}

fn main() {

    match logger::setup() {
        Ok(()) => log::debug!("logger is successfull initialized"),
        Err(e) => panic!("failed to initalize logger: {}", e)
    }

    let opts: Options = Options::from_args();
    log::info!("template file: {}", opts.template);
    log::info!("output file: {}", opts.output);
    log::info!("line count: {}", opts.count);
    log::info!("macro symbol: {}", opts.symbol);

    log::info!("read a template from '{}'", &opts.template);
    let template = match std::fs::read_to_string(&opts.template) {
        Ok(t) => t,
        Err(e) => {
            log::error!("failed to read a template: {}", e);
            quit_err(e)
        }
    };

    log::info!("create an output file '{}'", &opts.output);
    let output = match std::fs::File::create(&opts.output) {
        Ok(o) => o,
        Err(e) => {
            log::error!("failed to create an output file: {}", e);
            quit_err(e)
        }
    };

    let mut buffer = BufWriter::new(output);

    log::info!("parse the template");
    let generust = match generust::parse(&template, &opts.symbol) {
        Ok(g) => g,
        Err(e) => quit_err(e)
    };

    log::info!("start data generation");
    let mut p = 0;
    for i in 0..opts.count {
        match generust.generate(&mut buffer) {
            Ok(_) => {
                let n = 100 * i / opts.count;
                if n > p {
                    p = n;
                    log::debug!("{}%", p);
                }
            },
            Err(e) => {
                log::error!("failed to generate line {}: {}", i, e);
                quit_err(e)
            }
        }
    }

    log::info!("finish data generation");
    match buffer.flush() {
        Ok(_) => (),
        Err(e) => {
            log::error!("failed to flush output buffer: {}", e);
            quit_err(e)
        }
    }

}
