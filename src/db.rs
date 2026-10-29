use std::convert::Infallible;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use warp::Filter;

use crate::models;

// TODO: parametrize this function
pub async fn make_pool() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:bimage.db?mode=rwc")
        .await
        .expect("failed to open database");
    pool
}

pub fn with_pool(
    pool: SqlitePool,
) -> impl Filter<Extract = (SqlitePool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

pub async fn all_boards(pool: &SqlitePool) -> Result<Vec<models::Board>, sqlx::Error> {
    sqlx::query_as::<_, models::Board>(
        "select id, name, slug, description, active from boards order by name;",
    )
    .fetch_all(pool)
    .await
}

pub async fn board_by_id(pool: &SqlitePool, board_id: i64) -> Result<models::Board, sqlx::Error> {
    sqlx::query_as::<_, models::Board>(
        "select id, name, slug, description, active from boards where id = ? order by name;",
    )
    .bind(board_id)
    .fetch_one(pool)
    .await
}

pub async fn threads_by_board(
    pool: &SqlitePool,
    board_id: i64,
) -> Result<Vec<models::Thread>, sqlx::Error> {
    sqlx::query_as::<_, models::Thread>(
        "select id, board, created_at, bumped_at, active, image_id, title, body from models::Threads where board_id = ? and active order by bumped_at desc;"
    )
        .bind(board_id)
        .fetch_all(pool)
        .await
}
