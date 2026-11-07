use warp::Filter;

mod db;
mod errors;
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
    let main_page = warp::path::end()
        .and(db::with_pool(pool.clone()))
        .then(routes::boards);
    let bare_board = db::with_pool(pool.clone())
        .and(warp::path!(String))
        .then(routes::board);
    let thread = db::with_pool(pool.clone())
        .and(warp::path!(String / "threads" / i64))
        .then(routes::thread);

    let all_routes = warp::get().and(
        favicon
            .or(main_page)
            .or(admin)
            .or(healthz)
            .or(bare_board)
            .or(thread),
    );

    warp::serve(all_routes).run(([127, 0, 0, 1], 3030)).await;
}
