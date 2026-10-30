use sqlx::sqlite::SqlitePool;

use crate::models;

pub async fn all_boards(pool: &SqlitePool) -> Result<Vec<models::Board>, sqlx::Error> {
    sqlx::query_as::<_, models::Board>(
        "select id, name, slug, description, active from boards where active = 1 order by name;",
    )
    .fetch_all(pool)
    .await
}

pub async fn board_by_id(pool: &SqlitePool, board_id: i64) -> Result<models::Board, sqlx::Error> {
    sqlx::query_as::<_, models::Board>(
        "select id, name, slug, description, active from boards where id = ? and active = 1;",
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
        "select id, board_id, created_at, bumped_at, active, image_id, title, body \
        from threads where board_id = ? and active = 1 order by bumped_at desc;",
    )
    .bind(board_id)
    .fetch_all(pool)
    .await
}
