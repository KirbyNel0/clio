use std::io::Write;

use clap::Parser;
use clio::Clio;

/// Write everything from input to output.
#[derive(Parser)]
struct Args {
    /// Where to read from, may be `-` to read from standard input (required).
    from: Clio,
    /// Where to write to, may be omitted or `-` to write to standard output (optional).
    #[arg(short = 'o', default_value_t)]
    to: Clio,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    dbg!(&args.from);
    dbg!(&args.to);

    // read the input
    let input = args.from.reader().and_then(std::io::read_to_string)?;

    // write the output
    args.to
        .writer()
        .and_then(|mut writer| write!(writer, "{}", input))
}
