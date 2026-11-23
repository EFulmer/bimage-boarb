use sqlx::sqlite::SqlitePool;

use crate::errors::{Object, QueryError};
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
) -> Result<Vec<models::ThreadSummary>, sqlx::Error> {
    sqlx::query_as::<_, models::ThreadSummary>(
        "select id, board_id, created_at, bumped_at, active, title, \
        coalesce( \
            (select body from posts where thread_id = threads.id order by created_at, id limit 1), \
            '') \
        as first_post_body
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
        "select id, board_id, created_at, bumped_at, active, title \
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

/// Return the database's ID on successful creation of the thread, else a descriptive error; board
/// not found or internal DB error.
pub async fn make_thread(
    pool: &SqlitePool,
    new_thread: models::NewThread,
) -> Result<i64, QueryError> {
    let mut tx = pool.begin().await?;
    // This requires two inserts; one for the thread and one for its OP.
    let Some(new_thread_id) = sqlx::query_scalar(
        "insert into threads (board_id, title) \
        select id, ? from boards where slug = ? and active = 1 \
        returning id",
    )
    .bind(&new_thread.title)
    .bind(&new_thread.board_slug)
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Err(QueryError::NotFound(Object::Board(new_thread.board_slug)));
    };
    sqlx::query(
        "insert into posts (thread_id, screen_name, body) \
        values (?, ?, ?)",
    )
    .bind(new_thread_id)
    .bind(&new_thread.screen_name)
    .bind(&new_thread.body)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(new_thread_id)
}

/// Return the new reply's ID on successful creation, else a descriptive error; parent thread not
/// found or internal DB error.
pub async fn insert_reply(
    pool: &SqlitePool,
    board_slug: &str,
    new_post: models::NewPost,
) -> Result<i64, QueryError> {
    let mut tx = pool.begin().await?;

    let bumped = sqlx::query(
        "update threads set bumped_at = unixepoch() \
        where id = ? and active = 1
            and board_id = (select id from boards where slug = ? and active = 1)",
    )
    .bind(new_post.thread_id)
    .bind(board_slug)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    // If there was no thread to bump (we don't have sageing), then we need to check whether the
    // board exists at all.
    if bumped == 0 {
        tx.rollback().await?;
        return match board_by_slug(pool, board_slug).await {
            Ok(_) => Err(QueryError::NotFound(Object::Thread(new_post.thread_id))),
            Err(sqlx::Error::RowNotFound) => {
                Err(QueryError::NotFound(Object::Board(board_slug.to_owned())))
            }
            Err(e) => Err(QueryError::Db(e)),
        };
    }

    let new_post_id = sqlx::query_scalar(
        "insert into posts (thread_id, body, screen_name) \
        values (?, ?, ?) returning id",
    )
    .bind(new_post.thread_id)
    .bind(new_post.body)
    .bind(new_post.screen_name)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(new_post_id)
}
