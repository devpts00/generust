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

help:
	cargo run -- --help

format:
	cargo fmt

clippy:
	cargo clippy