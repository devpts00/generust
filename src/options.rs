use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Generust", author = "devpts00", about = "Data generator tool")]
pub struct Options {
    #[structopt(
        short,
        long,
        default_value = "10",
        help = "Number of records to generate"
    )]
    pub count: i32,

    #[structopt(
        short,
        long,
        default_value = r"\$",
        help = r"Macro start in regex to parse macros, e.g. '\$' to parse ${REC_NUM}, '@' to parse @{REC_NUM}"
    )]
    pub macro_start: String,

    #[structopt(
        short,
        long,
        default_value = ",",
        help = "Separator to split macros' arguments, e.g. ',' to split DATE_SEQ(2020-01-01,2020-02-01), '~' to split DATE_SEQ(2020-01-01~2020-02-01)"
    )]
    pub separator_args: String,

    #[structopt(short, long, default_value = "2", help = "Verbosity level from 0 to 5")]
    pub verbose: u8,
}
