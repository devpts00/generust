clean:
	cargo clean

check: format
	cargo check

build: format clippy
	cargo build

release: format clippy
	cargo build --release

test:
	cargo test -- --nocapture

run: build
	cat template.txt | target/debug/generust -c 5

run-stress: release
	cat template.txt | target/release/generust -c 1000000 -v 3 > output.dat

run-ucs: #release
	cat ucs.template | target/release/generust -c 1000000 -v 4 > ucs.output

run-test: release
	cat test.template | target/release/generust -c 10

help:
	cargo run -- --help

format:
	cargo fmt

clippy:
	cargo clippy