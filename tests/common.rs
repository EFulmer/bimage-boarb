/// Utilities for integration tests.
use sqlx::SqlitePool;

use bimage_boarb::db;

const DB_NAME: &str = ":memory:";

pub async fn setup() -> SqlitePool {
    let pool = db::make_pool(DB_NAME.to_string()).await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migration");
    pool
}

pub async fn create_board(pool: &SqlitePool) {
    sqlx::query(
        "insert into boards \
        (id, name, slug, description) \
        values (1, 'main', 'main', 'Just a test board.');",
    )
    .execute(pool)
    .await
    .unwrap();
}

pub async fn create_thread(pool: &SqlitePool) {
    sqlx::query(
        "insert into threads (id, board_id, title) \
        values (1, 1, 'testing a thread');
        ",
    )
    .execute(pool)
    .await
    .unwrap();
}
