clean:
	cargo clean

check: format
	cargo check

build: format
	cargo build

release:
	cargo build --release

test:
	cargo test -- --nocapture

run: build
	cargo run -- -c 1000 -t template.txt -o output.dat -v 3

run-stress: release
	cargo run --release -- -c 10000000 -t template.txt -o output.dat

help:
	cargo run -- --help

format:
	cargo fmt
