# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

bimage-boarb is an image sharing and commenting service (an image board: boards → threads → posts, with optional images) for a small group of friends. Rust (edition 2024), warp 0.4 for HTTP, maud 0.27 for server-rendered HTML, sqlx 0.8 on SQLite. Warp and maud were chosen for their functional style. There is no JS frontend: pages are server-rendered with plain forms (HTMX may be added later).

## Commands

```sh
make tools        # one-time: install sqlx-cli (sqlite only)
make build        # cargo build
make run          # cargo run — serves on http://127.0.0.1:3030
make create-db    # sqlx database create (uses DATABASE_URL from .env)
make migrate-db   # sqlx migrate run
make seed-db      # cargo run --bin seed — loads data/boards.csv and data/threads.csv
```

There are no tests yet. Use `cargo test <name>` once they exist; `cargo clippy` / `cargo fmt` for lint/format.

New migrations: `sqlx migrate add <name>`, run by hand. Don't put migration or `.env` generation in a repeatable Makefile target (it creates duplicates / overwrites `.env`).

Commit messages use NumPy-style prefixes: `ENH:`, `BUG:`, `BLD:`, `DOC:`, `RFR:` (combinable, e.g. `BUG/RFR: ...`). One fix or feature per branch.

## Architecture

- `src/main.rs` — opens the SQLite pool, runs embedded migrations (`sqlx::migrate!()` reads `migrations/` at compile time, so the server migrates itself on startup), builds warp filters, and combines them with `.or()` under `warp::get()`.
- `src/routes.rs` — handlers. Each returns a `warp::reply::Response` built from a maud `html!` template, mapping DB errors to 500/404 via `eprintln!` + `with_status`. The `thread` handler and its helpers (`threads()`, `posts()`) still render hardcoded fake data; its route is commented out in `main.rs`.
- `src/db.rs` — async query functions taking `&SqlitePool` and returning `Result<_, sqlx::Error>`, plus `with_pool`, a warp filter that injects a cloned pool (clones are cheap `Arc` handles; there is deliberately no global pool).
- `src/models.rs` — `sqlx::FromRow` structs mirroring the tables.
- `src/bin/seed.rs` — separate binary, and therefore its own crate root: it cannot import from `main.rs`. If binaries need to share types, split into `src/lib.rs` plus thin binaries using `bimage_boarb::...`. `default-run` in Cargo.toml keeps bare `cargo run` pointed at the server.

## Warp

- **Handler arguments follow filter order.** Each filter extracts a tuple; `.and()` concatenates them left to right, and the handler receives them as separate arguments. `db::with_pool(...).and(warp::path!("boards" / i64))` yields `(SqlitePool, i64)`, so `.then(routes::board)` works directly with `fn board(pool, board_id)`. Closure parameter names are only labels — a swapped pair still compiles if the types line up, which caused a past bug.
- `.map` takes a sync fn, `.then` an async fn, `.and_then` one returning `Result<_, Rejection>`. Mixing them up produces errors like "`Html<String>` is not a future" or "`impl Future` is not a `Reply`".
- When match arms return different reply types, call `.into_response()` on each (needs `use warp::Reply`).
- Error handling is currently a `match` per handler. The intended next step, once several routes share failure modes, is custom `Reject` types plus `.recover(...)`.

## Dependencies — gotchas

- **Do not enable maud's `warp` feature.** It pulls in warp 0.3, leaving two copies of warp and a misleading "trait `Reply` not implemented" error. Convert with `warp::reply::html(markup.into_string())`.
- sqlx-cli is pinned to `~0.8` to match the library major version (0.9 also needs a newer rustc than is installed). The sqlx feature is `sqlite`, not `sqlite3`.
- Queries use runtime `sqlx::query_as::<_, T>("...")` with explicitly named columns. The compile-time `query_as!` macros were deliberately deferred: they need `DATABASE_URL` at build time (or `cargo sqlx prepare` + a committed `.sqlx/`), and their SQLite nullability inference can disagree with the structs.

## Database

- **Schema changes go in a new file under `migrations/`**, never by editing an applied migration or a loose `.sql` file elsewhere.
- `FromRow` maps columns to fields **by name**, so `models.rs` names must match columns exactly (e.g. `threads.board_id`). Nullable columns must be `Option<T>` — decoding `NULL` into a plain `String` fails at runtime, not compile time.
- IDs are `i64`, never `u64` (sqlx doesn't map `u64` for SQLite).
- Timestamps are Unix epoch integers (`unixepoch()` defaults).
- The DB URL is hardcoded as `sqlite:bimage.db?mode=rwc` in `main.rs` and `db::make_pool` (`rwc` creates the file if missing), and as `sqlite:bimage.db` in `seed.rs` — relative to the working directory. `.env`'s `DATABASE_URL` is only used by sqlx-cli.
- Bumping is thread-level: a reply should insert the post and `update threads set bumped_at = unixepoch()` in one transaction. `posts.bumped_at` is vestigial and slated for removal.
- Open design question: whether a thread's image belongs on the thread or on its first post.

## Planned direction

Real queries for the thread route (`/boards/{id}/threads/{id}`), a shared maud `layout(title, body)` wrapper, form handlers for new threads and replies (redirect back after POST), and image uploads via `warp::multipart::form()` with a size cap, content-sniffed file type (not extension), and sha256 dedup into `images`. Later, possibly: `tracing` instead of `eprintln!`, `DATABASE_URL` read from the environment, and an `AppState` struct once there's more shared state than the pool.

`docker/` is an untracked work in progress (references Postgres, unconfirmed artifact path) and does not match the SQLite setup.
