use serde;
use sqlx;

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
    pub active: bool,
}

#[derive(sqlx::FromRow)]
pub struct Thread {
    pub id: i64,
    pub board_id: i64,
    pub created_at: i64,
    pub bumped_at: i64,
    pub active: bool,
    pub image_id: Option<i64>,
    pub title: String,
    pub body: String,
}

#[derive(sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub thread_id: i64,
    pub created_at: i64,
    pub active: bool,
    pub image_id: Option<i64>,
    pub screen_name: Option<String>,
    pub body: String,
    pub in_reply_to: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub struct Image {
    pub id: i64,
    pub uri: String,
    pub sha256: String,
    pub mime_type: String,
    pub original_name: Option<String>,
}
