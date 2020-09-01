mod generust;
mod options;
mod logger;

use std::io::{BufWriter, Write};
use options::Options;
use structopt::StructOpt;
use crate::generust::{Parser, GrError, GrResult};

fn quit<T>(code: Option<i32>) -> T {
    std::process::exit(code.unwrap_or_else(|| 1));
}

fn run(opts: Options) -> GrResult<()> {
    log::info!("read a template from '{}'", &opts.template);
    let template = std::fs::read_to_string(&opts.template)?;

    log::info!("create an output file '{}'", &opts.output);
    let output = std::fs::File::create(&opts.output)?;
    let mut buffer = BufWriter::new(output);

    log::info!("create template parser");
    let parser = Parser::new(&opts.symbol)?;

    log::info!("parse the template");
    let generust = parser.parse(&template)?;

    log::info!("start data generation");
    let mut p = 0;
    for i in 0..opts.count {
        generust.generate(&mut buffer)?;
        let n = 100 * i / opts.count;
        if n > p {
            p = n;
            log::debug!("{}%", p);
        }
    }

    log::info!("finish data generation");
    Ok(buffer.flush()?)
}

fn main() {

    let opts: Options = Options::from_args();

    match logger::setup(opts.verbose) {
        Ok(()) => log::debug!("logger is successfully initialized"),
        Err(e) => panic!("failed to initalize logger: {}", e)
    }

    log::info!("template file: {}", opts.template);
    log::info!("output file: {}", opts.output);
    log::info!("line count: {}", opts.count);
    log::info!("macro symbol: {}", opts.symbol);

    match run(opts) {
        Ok(_) => {
        },
        Err(err) => {
            log::error!("{}", err);
            match err {
                GrError::Io(err) => quit(err.raw_os_error()),
                _ => quit(None)
            }
        }
    }
}
