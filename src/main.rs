use std::env;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use bimage_boarb::db;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    let db_name = env::var("DB_NAME").unwrap_or("bimage.db".to_string());
    let pool = db::make_pool(db_name).await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migration");

    let all_routes = bimage_boarb::routes(pool);
    tracing::info!("listening on 127.0.0.1:3030");
    warp::serve(all_routes).run(([127, 0, 0, 1], 3030)).await;
}
