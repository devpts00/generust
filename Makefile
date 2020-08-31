build:
	cargo build

build-release:
	cargo build --release

test:
	cargo test -- --nocapture

run:
	cargo run -- -c 1000 -t template.txt -o output.txt

help:
	cargo run -- --help