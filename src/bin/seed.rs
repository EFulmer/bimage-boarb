use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;

#[derive(Deserialize)]
struct BoardRow {
    id: i64,
    name: String,
    slug: String,
    description: String,
}

#[derive(Deserialize)]
struct ThreadRow {
    id: i64,
    board_id: i64,
    created_at: i64,
    bumped_at: i64,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct PostRow {
    id: i64,
    thread_id: i64,
    created_at: i64,
    body: String,
    screen_name: String,
    in_reply_to: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: parametrize
    let pool = SqlitePoolOptions::new().connect("sqlite:bimage.db").await?;
    let mut reader = csv::Reader::from_path("data/boards.csv")?;

    for row in reader.deserialize::<BoardRow>() {
        let board = row?;
        sqlx::query(
            "insert or ignore into boards (id, name, slug, description) values (?, ?, ?, ?)",
        )
        .bind(board.id)
        .bind(board.name)
        .bind(board.slug)
        .bind(board.description)
        .execute(&pool)
        .await?;
    }

    let mut reader = csv::Reader::from_path("data/threads.csv")?;

    for row in reader.deserialize::<ThreadRow>() {
        let thread = row?;
        sqlx::query(
            "insert or ignore into threads (id, board_id, created_at, bumped_at, title, body) \
             values (?, ?, ?, ?, ?, ?)",
        )
        .bind(thread.id)
        .bind(thread.board_id)
        .bind(thread.created_at)
        .bind(thread.bumped_at)
        .bind(thread.title)
        .bind(thread.body)
        .execute(&pool)
        .await?;
    }

    let mut reader = csv::Reader::from_path("data/posts.csv")?;

    for row in reader.deserialize::<PostRow>() {
        let post = row?;
        sqlx::query(
            "insert or ignore into posts (id, thread_id, created_at, body, screen_name, in_reply_to) \
            values (?, ?, ?, ?, ?, ?)",
        )
        .bind(post.id)
        .bind(post.thread_id)
        .bind(post.created_at)
        .bind(post.body)
        .bind(post.screen_name)
        .bind(post.in_reply_to)
        .execute(&pool)
        .await?;
    }

    Ok(())
}
