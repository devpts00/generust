use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Generust", author = "devpts00", about = "Data generator tool")]
pub struct Options {
    #[structopt(short, long, default_value = "template.txt", help = "Template file")]
    pub template: String,
    #[structopt(short, long, default_value = "output.txt", help = "Output file")]
    pub output: String,
    #[structopt(short, long, default_value = "100", help = "Number of records to generate")]
    pub count: u32,
    #[structopt(short, long, default_value = r"\$", help = "Symbol to start macros in regex")]
    pub symbol: String,
    #[structopt(short, long, default_value = "2", help = "Verbosity level from 0 to 5")]
    pub verbose: u8,
}
