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
    for (name, entries) in &ds.entries {
        results.par_extend(
            entries
                .clone()
                .into_par_iter()
                .chunks(batch_size)
                .map(|entries| {
                    let entries = HashMap::from([(name.to_owned(), entries)]);
                    let ds = TorsionDriveResultCollection {
                        entries,
                        provenance: ds.provenance.clone(),
                        typ: ds.typ.clone(),
                    };
                    let json = serde_json::to_string(&ds).unwrap();
                    let script = script.replace("{json}", &json);
                    let mut cmd = Command::new("python");
                    let output = cmd.arg("-c").arg(&script).output().unwrap();

                    if !output.status.success() {
                        println!("{script}");
                        panic!("{}", String::from_utf8_lossy(&output.stderr));
                    }

                    let ds: TorsionDriveResultCollection =
                        serde_json::from_slice(&output.stdout).unwrap();
                    ds
                }),
        )
    }

    match results.len().cmp(&1) {
        // nothing happened, just return the original
        std::cmp::Ordering::Less => ds,
        // common case of one dataset
        std::cmp::Ordering::Equal => results.pop().unwrap(),
        // combine multiple sets of entries, warning on any provenance and type
        // mismatches. it's not clear how or if you can even combine these
        std::cmp::Ordering::Greater => {
            let mut results = results.into_iter();
            let mut ret = results.next().unwrap();
            for TorsionDriveResultCollection {
                entries,
                provenance,
                typ,
            } in results
            {
                if ret.provenance != provenance {
                    eprintln!("warning: provenance mismatch within datasets");
                }
                if ret.typ != typ {
                    eprintln!("warning: type mismatch within datasets");
                }
                // wow these are some bad names. the `entries` field is actually
                // HashMap<String, Vec<Entry>> because datasets are maps of a
                // hostname (or something) to a set of entries. So we have to
                // loop through this hashmap and extend the actual Vec<Entry>
                // for our single result. To complicate matters further, the
                // HashMap API has its own concept of an `Entry`, which we use
                // to either get a handle to existing Vec<OurEntry> or insert a
                // default (empty) Vec
                for (k, v) in entries {
                    let entry = ret.entries.entry(k).or_default();
                    entry.extend(v);
                }
            }
            ret
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use openff_toolkit::qcsubmit::results::{
        filters::{Filter, Filters},
        Provenance,
    };

    use super::*;

    #[test]
    fn round_trip() {
        let ds = TorsionDriveResultCollection::parse_file("testfiles/min.json")
            .unwrap();
        let script = read_to_string("scripts/filter_td.py").unwrap();
        let got = filter(ds.clone(), &script, 12);

        // the conversion to TorsionDriveResultCollection *in Python* is eating
        // the provenance, so just forget about it here. The provenance is
        // supposed to stack up, so we should get the input provenance (10
        // entries) chained with the additional filters in filter_td.py (6
        // more), but the input ones are disappearing, giving us only 6 in the
        // output
        use Filter::*;
        let want_prov = Provenance {
            applied_filters: Filters(vec![
                HydrogenBond {
                    method: "baker-hubbard".to_owned(),
                },
                RecordStatus {
                    status: "COMPLETE".to_owned(),
                },
                Connectivity { tolerance: 1.2 },
                UnperceivableStereo {
                    toolkits: vec!["openeye".to_owned(), "rdkit".to_owned()],
                },
                Element {
                    allowed_elements: vec![
                        "H".to_owned(),
                        "C".to_owned(),
                        "N".to_owned(),
                        "O".to_owned(),
                        "S".to_owned(),
                        "P".to_owned(),
                        "F".to_owned(),
                        "Cl".to_owned(),
                        "Br".to_owned(),
                    ],
                },
                Misc("ChargeCheckFilter".to_owned()),
            ]),
        };
        assert_eq!(got.entries, ds.entries);
        assert_eq!(got.provenance, want_prov);
        assert_eq!(got.typ, ds.typ);
    }
}
