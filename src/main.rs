use clap::Parser;
use fast_filter::filter;
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
}

fn main() {
    let cli = Cli::parse();

    ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build_global()
        .unwrap();

    let ds = TorsionDriveResultCollection::parse_file(&cli.input).unwrap();

    let got = filter(ds, cli.batch_size);

    // TODO I will probably want to collect the datasets themselves instead of
    // just entries and then combine them more intelligently out here.

    std::fs::write("output.json", &serde_json::to_string_pretty(&got).unwrap())
        .unwrap();
}
