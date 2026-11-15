/// Response from the `/healthz` endpoint; just a bool indicating whether or not the service is up at all.
#[derive(serde::Serialize)]
pub struct Healthz {
    pub ok: bool,
}

/// 1:1 correspondence with a row from the `boards` table; i.e. a board.
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

/// 1:1 correspondence with a row from the `threads` table; i.e. a thread.
#[derive(Debug, sqlx::FromRow)]
pub struct Thread {
    pub id: i64,
    #[expect(dead_code, reason = "used in queries, and usage will expand later")]
    pub board_id: i64,
    pub created_at: i64,
    #[expect(dead_code, reason = "present in schema, and usage will expand later")]
    pub bumped_at: i64,
    #[expect(
        dead_code,
        reason = "queries filter on active (missed by Rust's checks), plus admin will need to use it."
    )]
    pub active: bool,
    pub title: String,
}

/// 1:1 correspondence with a row from the `posts` table; i.e. a post.
#[derive(sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    #[expect(
        dead_code,
        reason = "used in queries (missed by Rust's checks), and usage will expand later"
    )]
    pub thread_id: i64,
    pub created_at: i64,
    #[expect(
        dead_code,
        reason = "queries filter on active (missed by Rust's checks), plus admin will need to use it."
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

/// A filled reply form accepted as the form body in a POST to `/{board_slug}/threads/{id}`. Not yet
/// validated.
#[derive(Debug, serde::Deserialize)]
pub struct ReplySubmission {
    pub screen_name: String,
    pub body: String,
}

/// A valid post yet to be inserted into the database; i.e. a validated `ReplySubmission` plus the
/// thread the reply is for.
#[derive(Debug)]
pub struct NewPost {
    pub thread_id: i64,
    pub screen_name: Option<String>,
    pub body: String,
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

/// Preview of a thread, for the `/{board_slug}` endpoint. Combines thread metadata with the first
/// post.
#[derive(sqlx::FromRow)]
pub struct ThreadSummary {
    pub id: i64,
    #[expect(dead_code, reason = "used in queries, and usage will expand later")]
    pub board_id: i64,
    pub created_at: i64,
    pub bumped_at: i64,
    #[expect(
        dead_code,
        reason = "queries filter on active (missed by Rust's checks), plus admin will need to use it."
    )]
    pub active: bool,
    pub title: String,
    pub first_post_body: String,
}

// Form limits, shared by the HTML `maxlength` attributes and server-side validation so the
// two can't drift apart. Counted in chars (`str::chars().count()`), not bytes.
pub const MAX_BODY_CHARS: usize = 10_000;
pub const MAX_NAME_CHARS: usize = 64;
pub const MAX_TITLE_CHARS: usize = 150;

pub const DEFAULT_SCREEN_NAME: &str = "Imbecile";
