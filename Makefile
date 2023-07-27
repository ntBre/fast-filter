clippy:
	cargo clippy --workspace --tests

test:
	cargo test

install:
	cargo install --path .

flame:
	CARGO_PROFILE_RELEASE_DEBUG=true \
	cargo flamegraph -- testfiles/min.json
	brave flamegraph.svg

profile:
	RUSTFLAGS='-g' cargo build --release
	valgrind --tool=callgrind --callgrind-out-file=callgrind.out    \
                --collect-jumps=yes --simulate-cache=yes                \
                ${BASE}/target/release/$(1)
