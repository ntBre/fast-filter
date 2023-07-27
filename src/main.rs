use std::{collections::HashMap, process::Command};

use clap::Parser;
use openff_toolkit::qcsubmit::results::TorsionDriveResultCollection;

use rayon::{prelude::*, ThreadPoolBuilder};

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

    let mut results = Vec::new();
    let ds_name = ds.entries.iter().next().unwrap().0;
    for (name, entries) in &ds.entries {
        // TODO accumulate results across entries just in case there is actually
        // more than one. That's what Lily does in the most recent script she
        // sent us: extend the entries of the first dataset with those in the
        // later ones
        results = entries
            .par_iter()
            .chunks(cli.batch_size)
            .map(|entries| {
                let map = HashMap::from([(name.to_owned(), entries)]);
                let json = serde_json::to_string(&map).unwrap();
                let script = include_str!("../scripts/filter_td.py")
                    .replace("{json}", &json);
                let mut cmd = Command::new("python");
                let output = cmd.arg("-c").arg(&script).output().unwrap();

                if !output.status.success() {
                    println!("{script}");
                    panic!("{}", String::from_utf8_lossy(&output.stderr));
                }

                let ds: TorsionDriveResultCollection =
                    serde_json::from_slice(&output.stdout).unwrap();
                ds.entries.into_values().flatten().collect::<Vec<_>>()
            })
            .flatten()
            .collect();
    }

    // TODO I will probably want to collect the datasets themselves instead of
    // just entries and then combine them more intelligently out here.
    let got = TorsionDriveResultCollection {
        entries: HashMap::from([(ds_name.clone(), results)]),
        ..ds
    };

    std::fs::write("output.json", &serde_json::to_string_pretty(&got).unwrap())
        .unwrap();
}
