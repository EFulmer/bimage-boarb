use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;

#[derive(Deserialize)]
struct BoardRow {
    name: String,
    slug: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new().connect("sqlite:bimage.db").await?;
    let mut reader = csv::Reader::from_path("data/boards.csv")?;

    for row in reader.deserialize::<BoardRow>() {
        let board = row?;
        sqlx::query("insert or ignore into boards (name, slug, description) values (?, ?, ?)")
            .bind(board.name)
            .bind(board.slug)
            .bind(board.description)
            .execute(&pool)
            .await?;
    }

    Ok(())
}
