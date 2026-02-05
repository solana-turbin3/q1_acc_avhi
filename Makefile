format:
	cd escrow-litesvm && cargo +nightly fmt --all
	cd whitelist-transfer-hook && cargo +nightly fmt --all

.PHONY: format
