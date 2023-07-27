# fast-filter

Running a sequence of filters on a `TorsionDriveResultCollection` or other
`openff-qcsubmit` result collection is painfully slow. This program uses my very
minimal Rust version of the
[openff-toolkit](https://github.com/ntBre/openff-toolkit) to parse one of these
collections from JSON, split it into individual entries, and run the filters on
each entry in a separate Python instance in parallel.

# Installation

To install `fast-filter` into `~/.cargo/bin`, simply run the following command:

```shell
make install
```

# Usage

`fast-filter` takes a required input dataset JSON file as an argument, as well
as a required Python script that performs the filter. It also has optional
arguments to set the batch size, number of threads, and output file:

``` shell
fast-filter input-dataset-to-filter.json -p filter-script.py \
	    [-o filtered-dataset.json] \
	    [-t number_of_threads] \
	    [-b batch_size]
```

The Python script can be pretty much anything, but it needs to include a couple
of key lines:

``` python
dataset = TorsionDriveResultCollection(
    entries=json.loads(r"""{json}""")["entries"]
)
...
print(dataset.json())
```

The first line is how Rust passes the data subsets back to Python; it replaces
the string `{json}` in the Python template file with a JSON literal. To ensure
proper escaping, you should wrap this in a Python raw string (the `r"""..."""`
above). The last line prints out the final dataset back in JSON format to be
parsed back into Rust.

# TODOs

Everything is hard-coded for now, so fix that. In particular,
- handle multiple collection types (at least Optimization in addition to
  TorsionDrive for my own use)
