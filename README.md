# fast-filter

Running a sequence of filters on a `TorsionDriveResultCollection` or other
`openff-qcsubmit` result collection is painfully slow. This program uses my very
minimal Rust version of the
[openff-toolkit](https://github.com/ntBre/openff-toolkit) to parse one of these
collections from JSON, split it into individual entries, and run the filters on
each entry in a separate Python instance in parallel.

# Usage

``` shell
cargo run --release
```

# TODOs

Everything is hard-coded for now, so fix that. In particular,
- take the name of the initial data set as input
- take the name of the output file as input (or just print to stdout)
- take the Python script template as input
- handle multiple collection types (at least Optimization in addition to
  TorsionDrive for my own use)
- experiment with a batch size instead of one entry per python call
