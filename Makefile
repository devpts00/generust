clean:
	cargo clean

check:
	cargo check

build:
	cargo build

release:
	cargo build --release

test:
	cargo test -- --nocapture

run:
	cargo run -- -c 1000 -t template.txt -o output.dat -v 2

run-stress: release
	cargo run --release -- -c 10000000 -t template.txt -o output.dat

help:
	cargo run -- --help