.PHONY: build test release
.DEFAULT_GOAL: build

build:
	cargo build

test: build
	cargo test

release: test
	cargo build --release
