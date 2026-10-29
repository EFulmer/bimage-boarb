# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

bimage-boarb is an image sharing and commenting service (an image board: boards → threads → posts, with optional images) for a small group of friends. Rust (edition 2024), warp for HTTP, maud for server-rendered HTML, sqlx on SQLite.

## Commands

```sh
make tools        # one-time: install sqlx-cli (sqlite only)
make build        # cargo build
make run          # cargo run — serves on http://127.0.0.1:3030
make create-db    # sqlx database create (uses DATABASE_URL from .env)
make migrate-db   # sqlx migrate run
make seed-db      # cargo run --bin seed — loads data/boards.csv into boards
```

There are no tests yet. Use `cargo test <name>` once they exist; `cargo clippy` / `cargo fmt` for lint/format.

Commit messages use NumPy-style prefixes: `ENH:`, `BUG:`, `BLD:`, `DOC:`, `RFR:` (combinable, e.g. `BUG/RFR: ...`). One fix or feature per branch.

## Architecture

- `src/main.rs` — opens the SQLite pool, runs embedded migrations (`sqlx::migrate!()` reads `migrations/` at compile time, so the server migrates itself on startup), builds warp filters, and combines them with `.or()` under `warp::get()`.
- `src/routes.rs` — handlers. Each returns a `warp::reply::Response` built from a maud `html!` template, mapping DB errors to 500/404 via `eprintln!` + `with_status`. The `thread` handler and its helpers (`threads()`, `posts()`) still render hardcoded fake data; its route is commented out in `main.rs`.
- `src/db.rs` — query functions (`sqlx::query_as` with runtime-checked SQL strings, not the compile-time `query!` macros) plus `with_pool`, a warp filter that injects a cloned `SqlitePool`.
- `src/models.rs` — `sqlx::FromRow` structs mirroring the tables.
- `src/bin/seed.rs` — separate binary (`default-run` in Cargo.toml keeps `cargo run` pointed at the server).

### Things to watch

- **Warp handler argument order follows filter order.** Whatever is extracted first in the `.and()` chain is the first handler parameter (e.g. `db::with_pool(...).and(warp::path!("boards" / i64))` → `routes::board(pool, board_id)`). Getting this wrong was a past bug.
- **Database path is hardcoded** as `sqlite:bimage.db` in `main.rs`, `db::make_pool`, and `seed.rs` (relative to the working directory); `.env`'s `DATABASE_URL` is only used by sqlx-cli. `*.db*` is gitignored.
- **Schema changes go in a new file under `migrations/`**, not by editing the existing migration. Keep `models.rs` field names in sync with column names — `FromRow` maps by name (e.g. the `threads` column is `board_id`).
- Timestamps are Unix epoch integers (`unixepoch()` defaults), stored as `i64`.
- `docker/` is an untracked work in progress (references Postgres, unconfirmed artifact path) and does not match the SQLite setup.
