clean:
	cargo clean

check: format clippy
	cargo check

build: format clippy
	cargo build

release: format clippy
	cargo build --release

test:
	cargo test -- --nocapture

run: build
	cargo run -- -c 1000 -t template.txt -o output.dat -v 3

run-stress: release
	cargo run --release -- -c 10000000 -t template.txt -o output.dat

run-ucs: build
	cargo run --release -- -c 1000000 -t ucs.template -o ucs.output -v 4

help:
	cargo run -- --help

format:
	cargo fmt

clippy:
	cargo clippy