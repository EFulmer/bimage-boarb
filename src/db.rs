use std::convert::Infallible;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use warp::Filter;

// TODO: parametrize this function
pub async fn make_pool() -> Pool<Sqlite> {
    SqlitePoolOptions::new()
        .connect("sqlite:bimage.db?mode=rwc")
        .await
        .expect("failed to open database")
}

pub fn with_pool(
    pool: SqlitePool,
) -> impl Filter<Extract = (SqlitePool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
