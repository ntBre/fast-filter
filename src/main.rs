use std::fs::read_to_string;

use clap::Parser;
use fast_filter::Filterer;
use openff_toolkit::qcsubmit::results::TorsionDriveResultCollection;
use rayon::ThreadPoolBuilder;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to operate on
    input: String,

    /// The number of entries to combine in one Python submission
    #[arg(short, long, default_value_t = 12)]
    batch_size: usize,

    /// The number of threads to use. 0 will detect the available threads
    /// automatically
    #[arg(short, long, default_value_t = 0)]
    threads: usize,

    /// The name of the template Python script to run in parallel.
    #[arg(short, long)]
    python_script: String,

    /// The optional name of the output file. Write the result to stdout if not
    /// provided
    #[arg(short, long)]
    output_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build_global()
        .unwrap();

    let ds = TorsionDriveResultCollection::parse_file(&cli.input).unwrap();

    let script = read_to_string(&cli.python_script).unwrap();

    let got = ds.filter(&script, cli.batch_size);

    let output = &serde_json::to_string_pretty(&got).unwrap();
    if let Some(out) = cli.output_file {
        std::fs::write(out, output).unwrap();
    } else {
        print!("{output}");
    }
}
