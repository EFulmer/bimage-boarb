use sqlx::sqlite::SqlitePool;

use crate::models;

pub async fn all_boards(pool: &SqlitePool) -> Result<Vec<models::Board>, sqlx::Error> {
    sqlx::query_as::<_, models::Board>(
        "select id, name, slug, description, active from boards where active = 1 order by name;",
    )
    .fetch_all(pool)
    .await
}

pub async fn board_by_slug(
    pool: &SqlitePool,
    board_slug: &str,
) -> Result<models::Board, sqlx::Error> {
    sqlx::query_as::<_, models::Board>(
        "select id, name, slug, description, active from boards where slug = ? and active = 1;",
    )
    .bind(board_slug)
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

pub async fn thread_by_id(
    pool: &SqlitePool,
    board_id: i64,
    thread_id: i64,
) -> Result<models::Thread, sqlx::Error> {
    sqlx::query_as::<_, models::Thread>(
        "select id, board_id, created_at, bumped_at, active, image_id, title, body \
        from threads where id = ? and board_id = ? and active = 1 limit 1;",
    )
    .bind(thread_id)
    .bind(board_id)
    .fetch_one(pool)
    .await
}

pub async fn posts_by_thread(
    pool: &SqlitePool,
    thread_id: i64,
) -> Result<Vec<models::Post>, sqlx::Error> {
    sqlx::query_as::<_, models::Post>(
        "select \
        id, thread_id, created_at, active, image_id, screen_name, body, in_reply_to \
        from posts where thread_id = ? and active = 1 \
        order by created_at, id;",
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await
}

pub async fn insert_reply(
    pool: &SqlitePool,
    board_slug: &str,
    thread_id: i64,
    new_post: models::NewPost,
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let bumped = sqlx::query(
        "update threads set bumped_at = unixepoch() \
        where id = ? and active = 1
            and board_id = (select id from boards where slug = ? and active = 1)",
    )
    .bind(thread_id)
    .bind(board_slug)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    tracing::info!(?bumped);
    if bumped == 0 {
        return Ok(None);
    }
    let insert = sqlx::query_as::<_, models::Post>(
        "insert into posts (thread_id, body, screen_name) \
        values (?, ?, ?) returning *",
    )
    .bind(thread_id)
    .bind(new_post.body)
    .bind(new_post.screen_name)
    .fetch_one(&mut *tx)
    .await;
    // tracing::info!(?insert);
    tx.commit().await?;
    match insert {
        Ok(post) => Ok(Some(post.id)),
        Err(e) => Err(e),
    }
}
