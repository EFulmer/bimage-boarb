#[derive(serde::Serialize)]
pub struct Healthz {
    pub ok: bool,
}

#[derive(sqlx::FromRow)]
pub struct Board {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub description: String,
    #[expect(
        dead_code,
        reason = "queries filter on active, plus admin will need to use it."
    )]
    pub active: bool,
}

#[derive(sqlx::FromRow)]
pub struct Thread {
    pub id: i64,
    #[expect(dead_code, reason = "used in queries, and usage will expand later")]
    pub board_id: i64,
    pub created_at: i64,
    pub bumped_at: i64,
    #[expect(
        dead_code,
        reason = "queries filter on active, plus admin will need to use it."
    )]
    pub active: bool,
    #[expect(
        dead_code,
        reason = "images haven't yet been implemented but they exist in the schema."
    )]
    pub image_id: Option<i64>,
    pub title: String,
    pub body: String,
}

#[derive(sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    #[expect(dead_code, reason = "used in queries, and usage will expand later")]
    pub thread_id: i64,
    pub created_at: i64,
    #[expect(
        dead_code,
        reason = "queries filter on active, plus admin will need to use it."
    )]
    pub active: bool,
    #[expect(
        dead_code,
        reason = "images haven't yet been implemented but they exist in the schema."
    )]
    pub image_id: Option<i64>,
    pub screen_name: Option<String>,
    pub body: String,
    pub in_reply_to: Option<i64>,
}

#[derive(sqlx::FromRow)]
#[expect(
    dead_code,
    reason = "images haven't yet been implemented but they exist in the schema."
)]
pub struct Image {
    pub id: i64,
    pub uri: String,
    pub sha256: String,
    pub mime_type: String,
    pub original_name: Option<String>,
}
