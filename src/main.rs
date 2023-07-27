use std::{collections::HashMap, process::Command};

use clap::Parser;
use openff_toolkit::qcsubmit::results::TorsionDriveResultCollection;

use rayon::prelude::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to operate on
    input: String,

    /// The number of entries to combine in one Python submission
    #[arg(short, long, default_value_t = 8)]
    batch_size: usize,
}

fn main() {
    let cli = Cli::parse();

    let ds = TorsionDriveResultCollection::parse_file(&cli.input).unwrap();

    let mut results = Vec::new();
    let ds_name = ds.entries.iter().next().unwrap().0;
    for (name, entries) in &ds.entries {
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

    let got = TorsionDriveResultCollection {
        entries: HashMap::from([(ds_name.clone(), results)]),
    };

    std::fs::write("output.json", &serde_json::to_string_pretty(&got).unwrap())
        .unwrap();
}
