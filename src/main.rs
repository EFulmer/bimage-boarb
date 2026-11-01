use warp::Filter;

mod db;
mod models;
mod queries;
mod routes;
mod templates;

#[tokio::main]
async fn main() {
    let pool = db::make_pool().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migration");

    let favicon = warp::path!("favicon.ico").map(|| warp::http::StatusCode::NO_CONTENT);
    let admin = warp::path!("admin").map(|| "This is the admin view. Most people aren't going to have access to what lives in these routes, which shouldn't be a surprise.");
    let healthz = warp::path!("healthz").map(routes::healthz);
    let boards = warp::path!("boards")
        .and(db::with_pool(pool.clone()))
        .then(routes::boards);
    let bare_board = db::with_pool(pool.clone())
        .and(warp::path!(String))
        .then(routes::board);
    // let thread = warp::path!("boards" / i64 / "threads" / i64)
    //     .and(db::with_pool(pool.clone()))
    //     .then(|pool, board_id, thread_id| routes::thread(pool, board_id, thread_id));

    let all_routes = warp::get().and(
        favicon.or(boards).or(admin).or(healthz).or(bare_board),
        // .or(thread)
    );

    warp::serve(all_routes).run(([127, 0, 0, 1], 3030)).await;
}
