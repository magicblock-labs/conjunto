DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

CARGO_TEST=nextest run
CARGO_TEST_NOCAP=nextest run --nocapture
$(if $(shell command -v cargo-nextest 2> /dev/null),,$(eval CARGO_TEST=test))
$(if $(shell command -v cargo-nextest 2> /dev/null),,$(eval CARGO_TEST_NOCAP=test -- --nocapture))

test:
	cargo $(CARGO_TEST)

test-log:
	cargo $(CARGO_TEST_NOCAP)

ci-build:
	cargo build

ci-clippy:
	cargo clippy --no-deps -- -D warnings

ci-test:
	cargo test \
		-p conjunto_addresses \
		-- --test-threads=1 --nocapture

.PHONY: test test-log ci-build ci-clippy ci-test
