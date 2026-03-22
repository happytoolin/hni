set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

build:
    cargo build

build-release:
    cargo build --release

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    HNI_NATIVE=false cargo test --all-targets --all-features

test-native:
    HNI_NATIVE=true cargo test --all-targets --all-features

test-all:
    node ./scripts/test-modes.mjs all

ci: fmt-check lint test test-native

bench:
    ./benchmark/run.sh

bench-compare:
    ./benchmark/run.sh --track=compare

bench-native:
    ./benchmark/run.sh --track=native

bench-runtime:
    ./benchmark/run.sh --track=runtime

bench-direct:
    ./benchmark/run.sh --track=direct

bench-profile:
    ./benchmark/profile.sh
