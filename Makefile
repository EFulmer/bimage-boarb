.PHONY: build run create_schema tools create-db migrate-db

# Dev installation:
tools:
	cargo install sqlx-cli --version '~0.8' --no-default-features --features sqlite

# Rust application:
build:
	cargo build

run:
	cargo run


# Database:
create-db:
	sqlx database create

migrate-db:
	sqlx migrate run

seed-db:
	cargo run --bin seed
