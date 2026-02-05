build:
	anchor build

test:
	anchor test

clean:
	cargo clean

format:
	cargo +nightly fmt --all

all:
	make clean && make build && make test

.PHONY: build test clean
