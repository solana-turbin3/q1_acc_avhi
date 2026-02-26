PHONY: build test fmt

build:
	cargo build	

test: 
	cargo test

check:
	cargo clippy

fmt: 
	cargo +nightly fmt --all

all:
	make build && make check && make test && make fmt
