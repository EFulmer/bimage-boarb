use sqlx::sqlite::SqlitePoolOptions;
use warp::Filter;

mod db;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:bimage.db?mode=rwc")
        .await
        .expect("failed to open database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migration");

    let boards = warp::path!("boards")
        .and(db::with_pool(pool.clone()))
        .then(routes::boards);
    let board = warp::path!("boards" / i64)
        .and(db::with_pool(pool.clone()))
        .then(|pool, board_id| routes::board(board_id, pool));
    // TODO: URL param for models::Thread ID
    let thread = warp::path!("boards" / "main" / "models::Threads" / "0").map(|| routes::thread());
    let admin = warp::path("admin").map(|| "This is the admin view. Most people aren't going to have access to what lives in these routes, which shouldn't be a surprise.");
    let healthz = warp::path("healthz").map(|| routes::healthz());

    let all_routes = warp::get().and(boards.or(board).or(thread).or(admin).or(healthz));

    warp::serve(all_routes).run(([127, 0, 0, 1], 3030)).await;
}
