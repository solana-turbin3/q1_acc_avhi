build:
	anchor build

deploy:
	anchor deploy --provider.cluster devnet

test:
	anchor test --skip-build --skip-deploy

clean:
	cargo clean

format:
	cargo +nightly fmt --all

check:
	cargo clippy

all:
	make clean && make build && make test

.PHONY: build test clean
