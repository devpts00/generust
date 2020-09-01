mod generust;
mod options;
mod logger;

use std::io::{BufWriter, Write};
use options::Options;
use structopt::StructOpt;
use crate::generust::Parser;

fn quit_err_code<T>(err: &dyn std::error::Error, code: Option<i32>) -> T {
    log::error!("error: {}", err);
    std::process::exit(code.unwrap_or_else(|| 1));
}

fn quit_err_io<T>(err: &std::io::Error) -> T {
    quit_err_code(err, err.raw_os_error())
}

fn quit_err<T>(err: &dyn std::error::Error) -> T {
    quit_err_code(err, Some(1))
}

fn main() {

    let opts: Options = Options::from_args();

    match logger::setup(opts.verbose) {
        Ok(()) => log::debug!("logger is successfull initialized"),
        Err(e) => panic!("failed to initalize logger: {}", e)
    }

    log::info!("template file: {}", opts.template);
    log::info!("output file: {}", opts.output);
    log::info!("line count: {}", opts.count);
    log::info!("macro symbol: {}", opts.symbol);

    log::info!("read a template from '{}'", &opts.template);
    let template = match std::fs::read_to_string(&opts.template) {
        Ok(t) => t,
        Err(e) => {
            log::error!("failed to read a template: {}", e);
            quit_err_io(&e)
        }
    };

    log::info!("create an output file '{}'", &opts.output);
    let output = match std::fs::File::create(&opts.output) {
        Ok(o) => o,
        Err(e) => {
            log::error!("failed to create an output file: {}", e);
            quit_err_io(&e)
        }
    };

    let mut buffer = BufWriter::new(output);



    log::info!("parse the template");
    let parser = match Parser::new(&opts.symbol) {
        Ok(p) => p,
        Err(e) => quit_err(&e)
    };

    let generust = match parser.parse(&template) {
        Ok(g) => g,
        Err(e) => quit_err_io(&e)
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
                quit_err_io(&e)
            }
        }
    }

    log::info!("finish data generation");
    match buffer.flush() {
        Ok(_) => (),
        Err(e) => {
            log::error!("failed to flush output buffer: {}", e);
            quit_err_io(&e)
        }
    }

}
