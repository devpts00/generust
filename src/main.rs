#![feature(try_trait)]

use std::io::{BufWriter, Read, Write};

use structopt::StructOpt;

use options::Options;

use crate::generust::{Error, Parser, Result};

mod generust;
mod logger;
mod options;

fn quit<T>(code: Option<i32>) -> T {
    std::process::exit(code.unwrap_or_else(|| 1));
}

fn run(opts: Options) -> Result<()> {
    log::debug!("read template");
    let mut template = String::new();
    std::io::stdin().lock().read_to_string(&mut template)?;

    log::debug!("parse template");
    let parser = Parser::new(&opts.macro_start, &opts.separator_args)?;
    let generust = parser.parse(&template)?;

    let stdout = std::io::stdout();
    let output = stdout.lock();
    let mut buffer = BufWriter::new(output);
    let mut p = 0;
    for i in 0..opts.count {
        generust.generate(i, &mut buffer)?;
        let n = 100 * i / opts.count;
        if n > p {
            p = n;
            log::debug!("progress: {}%", p);
        }
    }
    Ok(buffer.flush()?)
}

fn main() {
    let opts: Options = Options::from_args();

    match logger::setup(opts.verbose) {
        Ok(()) => log::debug!("logger is successfully initialized"),
        Err(e) => panic!("failed to initalize logger: {}", e),
    }

    log::info!("line count: {}", opts.count);
    log::info!("macro symbol: {}", opts.separator_args);
    log::info!("verbose level: {}", opts.verbose);

    match run(opts) {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
            match err {
                Error::Io(err) => quit(err.raw_os_error()),
                _ => quit(None),
            }
        }
    }
}
