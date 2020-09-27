DEBUG=target/debug/generust
RELEASE=target/release/generust

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

help:
	cargo run -- --help

format:
	cargo fmt

clippy:
	cargo clippy

make csv: release
	cd examples && cat template.csv | ../$(RELEASE) -c 5 | column -s, -t

make json: release
	cd examples && cat template.json | ../$(RELEASE) -c 1 | jq