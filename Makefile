#!/usr/bin/make -f
# Makefile for pingora-bench
# Can be executed directly: chmod +x Makefile
# Create symlinks to targets: ln -s Makefile build && ./build

# Detect if running via symlink and use basename as target
ifeq ($(MAKECMDGOALS),)
    SYMLINK_TARGET := $(shell basename "$(MAKEFILE_LIST)" 2>/dev/null)
    ifneq ($(SYMLINK_TARGET),Makefile)
        MAKECMDGOALS := $(SYMLINK_TARGET)
        .DEFAULT_GOAL := $(SYMLINK_TARGET)
    else
        .DEFAULT_GOAL := help
    endif
endif

.PHONY: help build release run test clean install bench quick-build quick-test

# Default target
help:
	@echo "Available targets:"
	@echo "  build    - Build the project in debug mode"
	@echo "  release  - Build the project in release mode"
	@echo "  run      - Run the project (requires --url argument)"
	@echo "  test     - Run tests"
	@echo "  clean    - Clean build artifacts"
	@echo "  install  - Install the binary to ~/.cargo/bin"
	@echo "  bench    - Run a quick benchmark test"
	@echo ""
	@echo "Usage examples:"
	@echo "  make build"
	@echo "  make run URL=https://example.com"
	@echo "  make bench URL=https://example.com DURATION=10"
	@echo ""
	@echo "Symlink trick:"
	@echo "  ln -s Makefile build && ./build"
	@echo "  ln -s Makefile release && ./release"

build:
	cargo build

release:
	cargo build --release

run:
	@if [ -z "$(URL)" ]; then \
		echo "Error: URL is required. Usage: make run URL=https://example.com"; \
		exit 1; \
	fi
	cargo run -- --url "$(URL)" $(ARGS)

test:
	cargo test

clean:
	cargo clean

install:
	cargo install --path .

bench:
	@if [ -z "$(URL)" ]; then \
		echo "Error: URL is required. Usage: make bench URL=https://example.com DURATION=10"; \
		exit 1; \
	fi
	@DURATION=$${DURATION:-10}; \
	cargo run --release -- --url "$(URL)" --duration $$DURATION $(ARGS)

# Example targets for symlink usage
quick-build: build
	@echo "Quick build completed!"

quick-test: test
	@echo "Tests completed!"