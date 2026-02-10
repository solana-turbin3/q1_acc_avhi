build:
	anchor build

test:
	anchor test

clean:
	cargo clean

format:
	cargo +nightly fmt --all

check:
	cargo clippy

all:
	make clean && make build && make test

.PHONY: build test clean
