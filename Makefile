ROOT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

# Escrow 
ESCROW_DIR := $(ROOT_DIR)/escrow-litesvm

escrow-build:
	cd $(ESCROW_DIR) && anchor build

escrow-test: escrow-build
	cd $(ESCROW_DIR) && cargo test -- --nocapture

escrow-clean:
	cd $(ESCROW_DIR) && cargo clean

# Whitelist Transfer Hook 
WHITELIST_DIR := $(ROOT_DIR)/whitelist-transfer-hook

whitelist-build:
	cd $(WHITELIST_DIR) && anchor build

whitelist-test:
	cd $(WHITELIST_DIR) && anchor test

whitelist-clean:
	cd $(WHITELIST_DIR) && cargo clean

# All 
build: escrow-build whitelist-build

test: escrow-test whitelist-test

clean: escrow-clean whitelist-clean

.PHONY: escrow-build escrow-test escrow-clean \
        whitelist-build whitelist-test whitelist-clean \
        build test clean
