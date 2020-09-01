mod generust;
mod options;
mod util;
mod logger;

use std::io::{BufWriter, Write};
use options::Options;
use structopt::StructOpt;

fn main() {

    match logger::setup() {
        Ok(()) => log::debug!("logger is successfull initialized"),
        Err(err) => panic!("failed to initalize logger: {}", err)
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
            util::quit_err(e)
        }
    };

    log::info!("create an output file '{}'", &opts.output);
    let output = match std::fs::File::create(&opts.output) {
        Ok(o) => o,
        Err(e) => {
            log::error!("failed to create an output file: {}", e);
            util::quit_err(e)
        }
    };

    let mut buffer = BufWriter::new(output);

    log::info!("parse the template");
    let generust = generust::parse(&template, &opts.symbol);

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
                util::quit_code(1)
            }
        }
    }

    log::info!("finish data generation");
    match buffer.flush() {
        Ok(_) => (),
        Err(e) => {
            log::error!("failed to flush output buffer: {}", e);
            util::quit_err(e)
        }
    }

}
