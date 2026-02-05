build:
	anchor build

test: build
	cargo test -- --nocapture

clean:
	cargo clean

format:
	cargo +nightly fmt --all

all: 
	make clean && make build && make test

.PHONY: build test clean
