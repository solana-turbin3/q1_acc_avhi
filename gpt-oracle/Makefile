build:
	anchor build

deploy:
	anchor deploy --provider.cluster devnet

test:
	yarn test:devnet

clean:
	rm -rf Cargo.lock && cargo clean

format:
	cargo +nightly fmt --all

check:
	cargo clippy

all:
	make clean && make build && make test

.PHONY: build test clean
