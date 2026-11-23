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
make create-db    # sqlx database create — creates the DB_NAME file (optional; migrate-db creates it too)
make migrate-db   # sqlx migrate run — creates DB_NAME if missing, then migrates it
make seed-db      # migrate-db, then cargo run --bin seed — loads data/boards.csv, data/threads.csv, data/posts.csv into DB_NAME
```

The Makefile exports `RUST_LOG ?= bimage_boarb=debug,info`, `DB_NAME ?= bimage.db`, and `DATABASE_URL ?= sqlite:${DB_NAME}?mode=rwc`, so one `DB_NAME` drives both the app and sqlx-cli; override any of them from the shell. `DATABASE_URL` is for sqlx-cli, `DB_NAME` for the app's code. Because the exported `DATABASE_URL` is already in the environment, sqlx-cli ignores `.env`'s value when run through make. If `DATABASE_URL` is already set in your shell, `?=` keeps it and it no longer follows `DB_NAME`. `mode=rwc` lets `migrate-db` (and so `seed-db`) create a missing file; without it SQLite defaults to `rw` and sqlx-cli fails on a new DB with "(code: 14) unable to open database file". sqlx-cli needs the `sqlite:` scheme; a bare path fails with "relative URL without a base". The server's built-in log default is `info`.

Tests:
- Unit tests live in `#[cfg(test)] mod tests` blocks at the bottom of the file they test (currently `templates.rs` and `routes.rs`, which covers form validation). They can reach private items via `use super::*;`.
- Integration tests are in `tests/integration.rs`, with helpers in `tests/common.rs` (`setup()` opens an in-memory DB through `db::make_pool(":memory:")` and runs migrations; `create_board`/`create_thread` insert fixtures). They drive `bimage_boarb::routes(pool)` with `warp::test::request()` and can only use the library's `pub` API (`db`, `routes`, `routes()`).
- Run one test with `cargo test <name>`. Test helpers use runtime `sqlx::query(...)`, not `query!`, so tests build without a database (see Dependencies).

New migrations: `sqlx migrate add <name>`, run by hand. Don't put migration or `.env` generation in a repeatable Makefile target (it creates duplicates / overwrites `.env`).

Commit messages use NumPy-style prefixes: `ENH:`, `BUG:`, `BLD:`, `DOC:`, `RFR:` (combinable, e.g. `BUG/RFR: ...`). One fix or feature per branch.

## Architecture

The package has a library crate (`src/lib.rs`, named `bimage_boarb`) and two thin binaries (`src/main.rs`, `src/bin/seed.rs`) that use it.

- `src/lib.rs` — declares every module exactly once (never also in `main.rs` — that compiles the file twice into two crates with incompatible types). Only `db` and `routes` are `pub`; the rest are private. Also defines `pub fn routes(pool) -> impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone`, which builds the whole app: GET routes under `warp::get()` (ending in a catch-all 404 route), POST routes under `warp::post()`, then `.recover(routes::handle_rejection)` and `.with(warp::trace::request())`. (`mod routes` and `fn routes` coexist because modules and functions are in separate namespaces.)
- `src/main.rs` — sets up `tracing`, opens the SQLite pool, runs embedded migrations (`sqlx::migrate!()` reads `migrations/` at compile time, so the server migrates itself on startup), and serves `bimage_boarb::routes(pool)`.
- `src/routes.rs` — handlers, plus form validation and `handle_rejection`. Each handler returns a `warp::reply::Response` built from a maud template, logging failures with `tracing` and mapping them to error pages (404/422/500, etc.).
- `src/queries.rs` — async query functions taking `&SqlitePool`. Read queries return `Result<_, sqlx::Error>` (a missing row is `sqlx::Error::RowNotFound`); the write queries `make_thread` and `insert_reply` return `Result<i64, QueryError>`.
- `src/db.rs` — `make_pool(db_name)`, `get_default_db_name()` (reads `DB_NAME`, defaults to `bimage.db`), and `with_pool`, a warp filter that injects a cloned pool (clones are cheap `Arc` handles; there is deliberately no global pool).
- `src/models.rs` — `sqlx::FromRow` structs for rows (`Thread`, `Post`, `ThreadSummary` for the board page), form input/insert types (`ReplySubmission` → `NewPost`), and shared limits (`MAX_BODY_CHARS`, `MAX_NAME_CHARS`, `DEFAULT_SCREEN_NAME`).
- `src/templates.rs` — maud templates as plain functions returning `Markup`, plus the reply form's view types (`ReplyForm`, `FieldError`). CSS is `static/style.css`, inlined into every page.
- `src/errors.rs` — `QueryError` (`NotFound(Object)` or `Db(sqlx::Error)`, with `impl From<sqlx::Error>` so `?` works in queries), returned by the write queries `make_thread` and `insert_reply`; `Object` names what was missing, with its key (`Board(String)` slug, `Thread(i64)` id). Error types own their data rather than borrowing, so they can travel up through `?` without lifetime parameters. Also the older per-route enums (`GetBoardError`, `GetThreadError`) used by the read paths, pending the refactor below.
- `src/bin/seed.rs` — separate binary using `src/db.rs`' helpers. It does **not** run migrations itself; `make seed-db` runs `migrate-db` first, but bare `cargo run --bin seed` needs an already-migrated DB. `default-run` in Cargo.toml keeps bare `cargo run` pointed at the server.

Visibility and `dead_code`: in a library, items reachable through the `pub` API never count as dead. Because `models`, `templates`, `errors` and `queries` are private, rustc still checks them, which is why `models.rs` keeps `#[expect(dead_code, reason = ...)]` on fields only read in SQL. Making one of those modules `pub` turns those into `unfulfilled_lint_expectations` errors under `make lint`.

## Warp

- **Handler arguments follow filter order.** Each filter extracts a tuple; `.and()` concatenates them left to right, and the handler receives them as separate arguments. `db::with_pool(...).and(warp::path!(String / "threads" / i64))` yields `(SqlitePool, String, i64)`, so `.then(routes::thread)` works directly with `fn thread(pool, board_slug, thread_id)`. Closure parameter names are only labels — a swapped pair still compiles if the types line up, which caused a past bug.
- `.map` takes a sync fn, `.then` an async fn, `.and_then` one returning `Result<_, Rejection>`. Mixing them up produces errors like "`Html<String>` is not a future" or "`impl Future` is not a `Reply`".
- When match arms return different reply types, call `.into_response()` on each (needs `use warp::Reply`).
- Rejections (unmatched method, malformed or oversized form bodies) are turned into error pages by `routes::handle_rejection`. It checks specific rejections **before** `MethodNotAllowed`: when both `.or()` branches reject, the combined rejection contains the GET side's `MethodNotAllowed` too, so checking it first turns every POST error into a 405.
- In-handler failures are still a `match` per handler; the `thread` and `reply` handlers duplicate the `GetThreadError` arms. A refactor is in progress: queries return `QueryError` instead of `Option`/`RowNotFound` (write queries done; read queries next), then handlers return `Result<Response, RouteError>` using `?`, with `impl Reply for RouteError` doing the logging and error page in one place. warp 0.4 has `impl<T: Reply, E: Reply> Reply for Result<T, E>`, so `.then(handler)` in `lib.rs` needs no change. Validation failures (422) stay an ordinary branch, not an error variant, since they re-render the page with the form.
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
- The server and seed both open `sqlite:{DB_NAME}?mode=rwc` via `db::make_pool`, relative to the working directory. `rwc` creates the file if it's missing; the server then migrates it, but bare `cargo run --bin seed` would fail on the empty file with "no such table". sqlx-cli uses `DATABASE_URL` instead, which the Makefile derives from `DB_NAME` (see Commands); `.env`'s `DATABASE_URL` only matters when running `sqlx` by hand. sqlx's in-memory URL (`:memory:`) gets a unique shared-cache name per pool, so all connections in one pool share one in-memory DB and each `make_pool(":memory:")` call (one per test) is isolated.
- A thread's opening post is its first row in `posts`, in `created_at, id` order. `threads` holds only the title and metadata; a thread's image belongs on its first post.
- Bumping is thread-level: a reply runs a guarded `update threads set bumped_at = unixepoch() where id = ? and active = 1 and board_id = (...)`, checks `rows_affected()` (0 means the thread is gone → 404), then inserts the post, all in one transaction. Replies always bump; there is no "sage" option.

## Planned direction

Replies (`POST /{slug}/threads/{id}`) and new threads (`POST /{slug}`) are done. Now in deployment: the `lib.rs` split and integration tests are in place, and the DB path comes from `DB_NAME`. Remaining: env config for host/port (bind `0.0.0.0` in the container), a container image, K3S manifests (1 replica, `Recreate`, RWO PVC), and SQLite backups. Then image uploads via `warp::multipart::form()` with a size cap, content-sniffed file type (not extension), and sha256 dedup into `images`. Separately: finish the error-handling refactor described under Warp. Later, possibly an `AppState` struct once there's more shared state than the pool.

`docker/` is an untracked work in progress (references Postgres, unconfirmed artifact path) and does not match the SQLite setup.
