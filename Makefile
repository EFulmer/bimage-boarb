.PHONY: fmt build run create_schema tools create-db migrate-db seed-db lint
export RUST_LOG ?= bimage_boarb=debug,info

# Dev installation:
tools:
	cargo install sqlx-cli --version '~0.8' --no-default-features --features sqlite

# Rust application:
build: fmt
	cargo build

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets -- -D warnings

run: fmt
	cargo run

test: fmt
	cargo test

# Database:
create-db:
	sqlx database create

migrate-db:
	sqlx migrate run

seed-db:
	cargo run --bin seed
