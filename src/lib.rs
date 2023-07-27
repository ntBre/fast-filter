use std::{collections::HashMap, process::Command};

use openff_toolkit::qcsubmit::results::TorsionDriveResultCollection;
use rayon::prelude::*;

/// The JSON contents of each entry in the dataset are replaced for the string
/// `{json}` in the template `script` file, so you need to include something
/// like the following to access it.
///
/// ```python
/// entries = dict(json.loads(r"""{json}"""))
/// dataset = TorsionDriveResultCollection(entries=entries)
/// ```
///
/// as well as print the dataset in JSON format at the end to get it back into
/// Rust:
///
/// ```python
/// print(dataset.json())
/// ```
pub fn filter(
    ds: TorsionDriveResultCollection,
    script: &str,
    batch_size: usize,
) -> TorsionDriveResultCollection {
    let mut results = Vec::new();
    let ds_name = ds.entries.iter().next().unwrap().0;
    for (name, entries) in &ds.entries {
        // TODO accumulate results across entries just in case there is actually
        // more than one. That's what Lily does in the most recent script she
        // sent us: extend the entries of the first dataset with those in the
        // later ones
        results = entries
            .par_iter()
            .chunks(batch_size)
            .map(|entries| {
                let map = HashMap::from([(name.to_owned(), entries)]);
                let json = serde_json::to_string(&map).unwrap();
                let script = script.replace("{json}", &json);
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

    TorsionDriveResultCollection {
        entries: HashMap::from([(ds_name.clone(), results)]),
        ..ds
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn round_trip() {
        let ds = TorsionDriveResultCollection::parse_file("testfiles/min.json")
            .unwrap();
        let script = read_to_string("scripts/filter_td.py").unwrap();
        let got = filter(ds.clone(), &script, 12);
        assert_eq!(got, ds);
    }
}
