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

csv: release
	cd examples && cat template.csv | ../$(RELEASE) -c 5 | column -s, -t

json: release
	cd examples && cat template.json | ../$(RELEASE) -c 1 | jq

stress: release
	cd examples && cat template.csv | ../$(RELEASE) -c 1000000 >/dev/null

memcheck: build
	cd examples && cat template.csv | valgrind --tool=memcheck --leak-check=full ../$(DEBUG) -c 10 >/dev/null

callgrind: build
	cd examples && cat template.csv | valgrind --tool=callgrind ../$(DEBUG) -c 10000 >/dev/null

cachegrind: release
	cd examples && cat template.csv | valgrind --tool=cachegrind ../$(DEBUG) -c 10000 >/dev/null
