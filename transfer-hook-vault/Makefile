build:
	anchor build

test: build
	cargo test -- --nocapture

clean:
	cargo clean

format:
	cargo +nightly fmt --all

check:
	cargo clippy

all: 
	make clean && make build && make test

.PHONY: build test clean
