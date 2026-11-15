# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

bimage-boarb is an image sharing and commenting service (an image board: boards → threads → posts, with optional images) for a small group of friends. Rust (edition 2024), warp 0.4 for HTTP, maud 0.27 for server-rendered HTML, sqlx 0.8 on SQLite. Warp and maud were chosen for their functional style. There is no JS frontend: pages are server-rendered with plain forms (HTMX may be added later).

## Commands

```sh
make tools        # one-time: install sqlx-cli (sqlite only)
make build        # cargo fmt + cargo build
make run          # cargo fmt + cargo run — serves on http://127.0.0.1:3030
make test         # cargo fmt + cargo test
make lint         # cargo clippy --all-targets -- -D warnings
make create-db    # sqlx database create (uses DATABASE_URL from .env)
make migrate-db   # sqlx migrate run
make seed-db      # cargo run --bin seed — loads data/boards.csv, data/threads.csv, data/posts.csv
```

Unit tests live in `#[cfg(test)]` modules (currently in `templates.rs`); run one with `cargo test <name>`. The Makefile exports `RUST_LOG ?= bimage_boarb=debug,info`; override it from the shell. The server's built-in default is `info`.

New migrations: `sqlx migrate add <name>`, run by hand. Don't put migration or `.env` generation in a repeatable Makefile target (it creates duplicates / overwrites `.env`).

Commit messages use NumPy-style prefixes: `ENH:`, `BUG:`, `BLD:`, `DOC:`, `RFR:` (combinable, e.g. `BUG/RFR: ...`). One fix or feature per branch.

## Architecture

- `src/main.rs` — sets up `tracing`, opens the SQLite pool, runs embedded migrations (`sqlx::migrate!()` reads `migrations/` at compile time, so the server migrates itself on startup), and builds the routes: GET routes under `warp::get()` (ending in a catch-all 404 route), POST routes under `warp::post()`, then `.recover(routes::handle_rejection)` and `.with(warp::trace::request())`.
- `src/routes.rs` — handlers, plus form validation and `handle_rejection`. Each handler returns a `warp::reply::Response` built from a maud template, logging failures with `tracing` and mapping them to error pages (404/422/500, etc.).
- `src/queries.rs` — async query functions taking `&SqlitePool` and returning `Result<_, sqlx::Error>`.
- `src/db.rs` — `make_pool`, and `with_pool`, a warp filter that injects a cloned pool (clones are cheap `Arc` handles; there is deliberately no global pool).
- `src/models.rs` — `sqlx::FromRow` structs for rows (`Thread`, `Post`, `ThreadSummary` for the board page), form input/insert types (`ReplySubmission` → `NewPost`), and shared limits (`MAX_BODY_CHARS`, `MAX_NAME_CHARS`, `DEFAULT_SCREEN_NAME`).
- `src/templates.rs` — maud templates as plain functions returning `Markup`, plus the reply form's view types (`ReplyForm`, `FieldError`). CSS is `static/style.css`, inlined into every page.
- `src/errors.rs` — per-route error enums (`BoardRouteError`, `ThreadRouteError`).
- `src/bin/seed.rs` — separate binary, and therefore its own crate root: it cannot import from `main.rs`. If binaries need to share types, split into `src/lib.rs` plus thin binaries using `bimage_boarb::...`. `default-run` in Cargo.toml keeps bare `cargo run` pointed at the server.

## Warp

- **Handler arguments follow filter order.** Each filter extracts a tuple; `.and()` concatenates them left to right, and the handler receives them as separate arguments. `db::with_pool(...).and(warp::path!(String / "threads" / i64))` yields `(SqlitePool, String, i64)`, so `.then(routes::thread)` works directly with `fn thread(pool, board_slug, thread_id)`. Closure parameter names are only labels — a swapped pair still compiles if the types line up, which caused a past bug.
- `.map` takes a sync fn, `.then` an async fn, `.and_then` one returning `Result<_, Rejection>`. Mixing them up produces errors like "`Html<String>` is not a future" or "`impl Future` is not a `Reply`".
- When match arms return different reply types, call `.into_response()` on each (needs `use warp::Reply`).
- Rejections (unmatched method, malformed or oversized form bodies) are turned into error pages by `routes::handle_rejection`. It checks specific rejections **before** `MethodNotAllowed`: when both `.or()` branches reject, the combined rejection contains the GET side's `MethodNotAllowed` too, so checking it first turns every POST error into a 405.
- In-handler failures are still a `match` per handler; the `thread` and `reply` handlers duplicate the `ThreadRouteError` arms. The intended next step is one error enum per route plus `?`.
- `warp::redirect::*` takes an `http::Uri`, which silently drops `#fragment`s. For redirects to `#p{id}`, build the 303 with `warp::reply::with_header(StatusCode::SEE_OTHER, "location", ...)`.

## Dependencies — gotchas

- **Do not enable maud's `warp` feature.** It pulls in warp 0.3, leaving two copies of warp and a misleading "trait `Reply` not implemented" error. Convert with `warp::reply::html(markup.into_string())`.
- sqlx-cli is pinned to `~0.8` to match the library major version (0.9 also needs a newer rustc than is installed). The sqlx feature is `sqlite`, not `sqlite3`.
- Queries use runtime `sqlx::query_as::<_, T>("...")` with explicitly named columns. The compile-time `query_as!` macros were deliberately deferred: they need `DATABASE_URL` at build time (or `cargo sqlx prepare` + a committed `.sqlx/`), and their SQLite nullability inference can disagree with the structs.

## Database

- **Schema changes go in a new file under `migrations/`**, never by editing an applied migration or a loose `.sql` file elsewhere.
- `FromRow` maps columns to fields **by name**, so `models.rs` names must match columns exactly (e.g. `threads.board_id`). Nullable columns must be `Option<T>` — decoding `NULL` into a plain `String` fails at runtime, not compile time.
- IDs are `i64`, never `u64` (sqlx doesn't map `u64` for SQLite).
- Timestamps are Unix epoch integers (`unixepoch()` defaults).
- The DB URL is hardcoded as `sqlite:bimage.db?mode=rwc` in `db::make_pool` (`rwc` creates the file if missing), and as `sqlite:bimage.db` in `seed.rs` — relative to the working directory. `.env`'s `DATABASE_URL` is only used by sqlx-cli.
- A thread's opening post is its first row in `posts`, in `created_at, id` order. `threads` holds only the title and metadata; a thread's image belongs on its first post.
- Bumping is thread-level: a reply runs a guarded `update threads set bumped_at = unixepoch() where id = ? and active = 1 and board_id = (...)`, checks `rows_affected()` (0 means the thread is gone → 404), then inserts the post, all in one transaction. Replies always bump; there is no "sage" option.

## Planned direction

Next: a new-thread form at `POST /{slug}` (insert the thread and its first post in one transaction, then redirect), reusing `NewPost`. Then deployment (env config for host/port/DB path, container), and image uploads via `warp::multipart::form()` with a size cap, content-sniffed file type (not extension), and sha256 dedup into `images`. Later, possibly: `DATABASE_URL` read from the environment and an `AppState` struct once there's more shared state than the pool.

`docker/` is an untracked work in progress (references Postgres, unconfirmed artifact path) and does not match the SQLite setup.
