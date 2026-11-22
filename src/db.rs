use std::convert::Infallible;
use std::env;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use warp::Filter;

pub async fn make_pool(db_name: String) -> Pool<Sqlite> {
    let connection_string = format!("sqlite:{db_name}?mode=rwc");
    SqlitePoolOptions::new()
        .connect(&connection_string)
        .await
        .expect("failed to open database")
}

pub fn with_pool(
    pool: SqlitePool,
) -> impl Filter<Extract = (SqlitePool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

pub fn get_default_db_name() -> String {
    env::var("DB_NAME").unwrap_or("bimage.db".to_string())
}
