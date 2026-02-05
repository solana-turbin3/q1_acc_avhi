build:
	anchor build

test: build
	cargo test -- --nocapture

clean:
	cargo clean

format:
	cargo +nightly fmt --all

.PHONY: build test clean
