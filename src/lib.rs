pub mod db;
mod errors;
mod models;
mod queries;
pub mod routes;
mod templates;

use sqlx::sqlite::SqlitePool;
use warp::Filter;

pub fn routes(
    pool: SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = std::convert::Infallible> + Clone {
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
    let not_found = warp::path::full().map(routes::page_not_found);

    let get_routes = warp::get().and(
        favicon
            .or(main_page)
            .or(admin)
            .or(healthz)
            .or(bare_board)
            .or(thread)
            .or(not_found),
    );

    let reply = db::with_pool(pool.clone())
        .and(warp::body::content_length_limit(128 * 1024))
        .and(warp::path!(String / "threads" / i64))
        .and(warp::body::form())
        .then(routes::reply);
    let new_thread = db::with_pool(pool.clone())
        .and(warp::body::content_length_limit(128 * 1024))
        .and(warp::path!(String))
        .and(warp::body::form())
        .then(routes::new_thread);

    let post_routes = warp::post().and(reply.or(new_thread));

    get_routes
        .or(post_routes)
        .recover(routes::handle_rejection)
        .with(warp::trace::request())
}
