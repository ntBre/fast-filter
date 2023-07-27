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
- handle multiple collection types (at least Optimization in addition to
  TorsionDrive for my own use)
