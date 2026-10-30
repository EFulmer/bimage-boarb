.PHONY: fmt build run create_schema tools create-db migrate-db seed-db

# Dev installation:
tools:
	cargo install sqlx-cli --version '~0.8' --no-default-features --features sqlite

# Rust application:
build: fmt
	cargo build

fmt:
	cargo fmt

run: fmt
	cargo run


# Database:
create-db:
	sqlx database create

migrate-db:
	sqlx migrate run

seed-db:
	cargo run --bin seed
