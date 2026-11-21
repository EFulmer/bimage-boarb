use std::convert::Infallible;

use sqlx::{Error, SqlitePool};
use warp::{
    Rejection, Reply,
    filters::body::BodyDeserializeError,
    http::StatusCode,
    path::FullPath,
    reject::{MethodNotAllowed, PayloadTooLarge, UnsupportedMediaType},
    reply::Response,
};

use crate::{errors, models, queries, templates};

const SCREEN_NAME_TOO_LONG_ERROR: &str = "Your screen name is too long.";
const TITLE_TOO_LONG_ERROR: &str = "Your post's title is too long.";
const TITLE_EMPTY_ERROR: &str = "Post titles cannot be empty";
const POST_TOO_LONG_ERROR: &str = "Your post is too long.";
const POST_EMPTY_ERROR: &str = "Your post cannot be empty.";

pub fn healthz() -> impl warp::Reply {
    let resp_body = models::Healthz { ok: true };
    warp::reply::with_status(warp::reply::json(&resp_body), StatusCode::OK)
}

#[tracing::instrument(skip(pool))]
pub async fn boards(pool: SqlitePool) -> Response {
    match queries::all_boards(&pool).await {
        Ok(boards) => {
            warp::reply::html(templates::boards_page(&boards).into_string()).into_response()
        }
        Err(error) => {
            tracing::error!(?error, "all_boards query failed");
            internal_server_error_page()
        }
    }
}

#[tracing::instrument(skip(pool))]
pub async fn board(pool: SqlitePool, board_slug: String) -> Response {
    match get_board(&pool, &board_slug, &templates::ThreadForm::default()).await {
        Ok(board_template) => warp::reply::html(board_template.into_string()).into_response(),
        Err(errors::BoardRouteError::NotFound(board_slug)) => {
            tracing::warn!("board not present or inactive in database");
            board_not_found(&board_slug)
        }
        Err(errors::BoardRouteError::Db(error)) => {
            tracing::error!(?error, "query for board failed");
            internal_server_error_page()
        }
    }
}

#[tracing::instrument(skip(pool))]
pub async fn thread(pool: SqlitePool, board_slug: String, thread_id: i64) -> Response {
    match get_thread(
        &pool,
        &board_slug,
        thread_id,
        &templates::ReplyForm::default(),
    )
    .await
    {
        Ok(thread_template) => warp::reply::html(thread_template.into_string()).into_response(),
        Err(errors::ThreadRouteError::BoardNotFound(board_slug)) => {
            tracing::warn!("board not present or inactive in database");
            board_not_found(&board_slug)
        }
        Err(errors::ThreadRouteError::ThreadNotFound(thread_id)) => {
            tracing::warn!("thread not present or inactive in database");
            thread_not_found(thread_id)
        }
        Err(errors::ThreadRouteError::Db(error)) => {
            tracing::error!(?error, "query for thread or board failed");
            internal_server_error_page()
        }
    }
}

#[tracing::instrument(skip(pool))]
pub async fn new_thread(
    pool: SqlitePool,
    board_slug: String,
    form: models::ThreadSubmission,
) -> Response {
    tracing::debug!(?board_slug, ?form, "got");
    match validate_thread_fields(&board_slug, &form) {
        Ok(thread) => match queries::make_thread(&pool, thread).await {
            Ok(Some(thread_id)) => {
                tracing::info!(?thread_id, "new thread made");
                let location = format!("/{board_slug}/threads/{thread_id}");
                warp::reply::with_header(StatusCode::SEE_OTHER, "location", location)
                    .into_response()
            }
            Ok(None) => {
                tracing::warn!("board not present or inactive in database");
                board_not_found(&board_slug)
            }
            Err(error) => {
                tracing::error!(?error, "failed to create thread");
                internal_server_error_page()
            }
        },
        Err(errors) => {
            tracing::info!(?errors, "errors in new thread form");
            let rendered_form = templates::ThreadForm {
                title: &form.title,
                screen_name: &form.screen_name,
                body: &form.body,
                errors: &errors,
            };
            match get_board(&pool, &board_slug, &rendered_form).await {
                Ok(board_template) => warp::reply::with_status(
                    warp::reply::html(board_template.into_string()),
                    StatusCode::UNPROCESSABLE_ENTITY,
                )
                .into_response(),
                Err(errors::BoardRouteError::NotFound(board_slug)) => {
                    tracing::warn!("board not found or inactive in database");
                    board_not_found(&board_slug)
                }
                Err(errors::BoardRouteError::Db(error)) => {
                    tracing::error!(?error, "query for board failed");
                    internal_server_error_page()
                }
            }
        }
    }
}

#[tracing::instrument]
fn validate_thread_fields(
    board_slug: &str,
    form: &models::ThreadSubmission,
) -> Result<models::NewThread, Vec<templates::FieldError>> {
    let mut errors = vec![];
    if let Err(screen_name_error) = validate_screen_name(&form.screen_name) {
        errors.push(screen_name_error);
    }

    if let Err(title_error) = validate_thread_title(&form.title) {
        errors.push(title_error);
    }

    if let Err(post_error) = validate_post_body(&form.body) {
        errors.push(post_error);
    }

    if errors.is_empty() {
        let screen_name = if form.screen_name.trim().is_empty() {
            None
        } else {
            Some(form.screen_name.trim().to_string())
        };
        Ok(models::NewThread {
            board_slug: board_slug.to_string(),
            title: form.title.trim().to_string(),
            screen_name,
            body: form.body.trim().to_string(),
        })
    } else {
        Err(errors)
    }
}

#[tracing::instrument(skip(pool))]
pub async fn reply(
    pool: SqlitePool,
    board_slug: String,
    thread_id: i64,
    form: models::ReplySubmission,
) -> Response {
    // TODO: at some point it might be worth flattening this out via its own enum type encoding the
    // possible ways it can fail (PostReplyStatus::Ok(NewPost), BoardNotFound, ThreadNotFound,
    // DBError(sqlx::Error))
    match validate_reply_fields(thread_id, &form) {
        Ok(new_post) => {
            tracing::debug!(?new_post, "form submission valid, record to insert; ");
            match queries::insert_reply(&pool, &board_slug, new_post).await {
                Ok(Some(post_id)) => {
                    let location = format!("/{board_slug}/threads/{thread_id}#p{post_id}");
                    tracing::debug!(
                        ?post_id,
                        ?location,
                        "it succeeded in inserting the reply! (n64 voice)"
                    );
                    warp::reply::with_header(
                        // A 303 SEE OTHER will avoid form resubmission, i.e. duplicate replying.
                        StatusCode::SEE_OTHER,
                        "location",
                        location,
                    )
                    .into_response()
                }
                Ok(None) => {
                    tracing::warn!("thread not present or inactive in database");
                    thread_not_found(thread_id)
                }
                Err(error) => {
                    tracing::error!(?error, "failed to insert reply");
                    internal_server_error_page()
                }
            }
        }
        Err(errors) => {
            tracing::debug!(?errors, "errors in form; ");
            let rendered_form = templates::ReplyForm {
                screen_name: &form.screen_name,
                body: &form.body,
                errors: &errors,
            };
            // TODO: eliminate repetition.
            match get_thread(&pool, &board_slug, thread_id, &rendered_form).await {
                Ok(thread_template) => warp::reply::with_status(
                    warp::reply::html(thread_template.into_string()),
                    StatusCode::UNPROCESSABLE_ENTITY,
                )
                .into_response(),
                Err(errors::ThreadRouteError::BoardNotFound(board_slug)) => {
                    tracing::warn!("board not present or inactive in database");
                    board_not_found(&board_slug)
                }
                Err(errors::ThreadRouteError::ThreadNotFound(thread_id)) => {
                    tracing::warn!("thread not present or inactive in database");
                    thread_not_found(thread_id)
                }
                Err(errors::ThreadRouteError::Db(error)) => {
                    tracing::error!(?error, "query for thread or board failed");
                    internal_server_error_page()
                }
            }
        }
    }
}

#[tracing::instrument]
fn validate_reply_fields(
    thread_id: i64,
    form: &models::ReplySubmission,
) -> Result<models::NewPost, Vec<templates::FieldError>> {
    let mut errors = vec![];

    if let Err(screen_name_error) = validate_screen_name(&form.screen_name) {
        errors.push(screen_name_error);
    }

    if let Err(post_error) = validate_post_body(&form.body) {
        errors.push(post_error);
    }

    if errors.is_empty() {
        let screen_name = if form.screen_name.trim().is_empty() {
            None
        } else {
            Some(form.screen_name.trim().to_string())
        };
        Ok(models::NewPost {
            thread_id,
            screen_name,
            body: form.body.trim().to_string(),
        })
    } else {
        Err(errors)
    }
}

fn validate_screen_name(screen_name: &str) -> Result<(), templates::FieldError> {
    if screen_name.trim().chars().count() > models::MAX_NAME_CHARS {
        Err(templates::FieldError {
            field: "screen_name",
            message: SCREEN_NAME_TOO_LONG_ERROR.to_string(),
        })
    } else {
        Ok(())
    }
}

fn validate_post_body(post_body: &str) -> Result<(), templates::FieldError> {
    if post_body.trim().chars().count() > models::MAX_BODY_CHARS {
        Err(templates::FieldError {
            field: "body",
            message: POST_TOO_LONG_ERROR.to_string(),
        })
    } else if post_body.trim().is_empty() {
        Err(templates::FieldError {
            field: "body",
            message: POST_EMPTY_ERROR.to_string(),
        })
    } else {
        Ok(())
    }
}

fn validate_thread_title(thread_title: &str) -> Result<(), templates::FieldError> {
    if thread_title.trim().chars().count() > models::MAX_TITLE_CHARS {
        Err(templates::FieldError {
            field: "title",
            message: TITLE_TOO_LONG_ERROR.to_string(),
        })
    } else if thread_title.trim().is_empty() {
        Err(templates::FieldError {
            field: "title",
            message: TITLE_EMPTY_ERROR.to_string(),
        })
    } else {
        Ok(())
    }
}

async fn get_board(
    pool: &SqlitePool,
    board_slug: &str,
    thread_form: &templates::ThreadForm<'_>,
) -> Result<maud::Markup, errors::BoardRouteError> {
    let board = queries::board_by_slug(pool, board_slug)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => errors::BoardRouteError::NotFound(board_slug.to_string()),
            e => errors::BoardRouteError::Db(e),
        })?;
    let threads = queries::threads_by_board(pool, board.id)
        .await
        .map_err(errors::BoardRouteError::Db)?;
    Ok(templates::board_page(&board, &threads, thread_form))
}

async fn get_thread(
    pool: &SqlitePool,
    board_slug: &str,
    thread_id: i64,
    reply_form: &templates::ReplyForm<'_>,
) -> Result<maud::Markup, errors::ThreadRouteError> {
    let board = queries::board_by_slug(pool, board_slug)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => errors::ThreadRouteError::BoardNotFound(board_slug.to_string()),
            e => errors::ThreadRouteError::Db(e),
        })?;
    let thread = queries::thread_by_id(pool, board.id, thread_id)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => errors::ThreadRouteError::ThreadNotFound(thread_id),
            e => errors::ThreadRouteError::Db(e),
        })?;
    let posts = queries::posts_by_thread(pool, thread.id)
        .await
        .map_err(errors::ThreadRouteError::Db)?;
    Ok(templates::thread_page(&board, &thread, &posts, reply_form))
}

pub fn internal_server_error_page() -> Response {
    error_page_with_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error.",
        "The board is having issues. The specifics have been logged for an admin to fix.",
    )
}

#[tracing::instrument]
pub async fn handle_rejection(err: Rejection) -> Result<Response, Infallible> {
    if err.find::<BodyDeserializeError>().is_some() {
        tracing::warn!("malformed request body");
        Ok(error_page_with_response(
            StatusCode::BAD_REQUEST,
            "Bad request",
            "Unexpected error submitting your reply. \
            The specifics have been logged for an admin to fix.",
        ))
    } else if err.find::<PayloadTooLarge>().is_some() {
        tracing::warn!("request payload too large");
        Ok(error_page_with_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "Payload too large",
            "Your reply was too long.",
        ))
    } else if err.find::<UnsupportedMediaType>().is_some() {
        tracing::warn!("unsupported media type");
        Ok(error_page_with_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported media type",
            "Unexpected error submitting your reply. \
            The specifics have been logged for an admin to fix.",
        ))
    } else if err.find::<MethodNotAllowed>().is_some() {
        tracing::warn!("method not allowed");
        Ok(error_page_with_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "Method not allowed",
            "An invalid method was sent to an endpoint. \
            The specifics have been logged for an admin to fix.",
        ))
    } else {
        tracing::error!("error handling request");
        Ok(internal_server_error_page())
    }
}

#[tracing::instrument]
pub fn page_not_found(path: FullPath) -> Response {
    not_found_page(
        "Page",
        path.as_str(),
        &format!(
            "A link brought you to {}, which isn't a page that exists \
                    on this site. Details have been logged for an admin to look into.",
            path.as_str(),
        ),
    )
}

fn board_not_found(board_name: &str) -> Response {
    not_found_page(
        "Board",
        board_name,
        &format!(
            "The board {board_name} doesn't exist. \
            Maybe it was misspelled, renamed, or hasn't been created yet?"
        ),
    )
}

fn thread_not_found(thread_id: i64) -> Response {
    not_found_page(
        "Thread",
        &thread_id.to_string(),
        "This thread is unavailable... \
                        or maybe it was never available in the first place? Much to think about.",
    )
}

fn not_found_page(item_type: &str, item_name: &str, message: &str) -> Response {
    error_page_with_response(
        StatusCode::NOT_FOUND,
        &format!("{item_type} {item_name} not found!"),
        message,
    )
}

fn error_page_with_response(status: StatusCode, heading: &str, message: &str) -> Response {
    warp::reply::with_status(
        warp::reply::html(templates::error_page(status.as_u16(), heading, message).into_string()),
        status,
    )
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::{MAX_BODY_CHARS, MAX_NAME_CHARS, MAX_TITLE_CHARS};

    /// Which fields failed, in the order the validator reported them.
    fn failed_fields(errors: &[templates::FieldError]) -> Vec<&'static str> {
        errors.iter().map(|e| e.field).collect()
    }

    fn reply(screen_name: &str, body: &str) -> models::ReplySubmission {
        models::ReplySubmission {
            screen_name: screen_name.to_string(),
            body: body.to_string(),
        }
    }

    fn thread(title: &str, screen_name: &str, body: &str) -> models::ThreadSubmission {
        models::ThreadSubmission {
            title: title.to_string(),
            screen_name: screen_name.to_string(),
            body: body.to_string(),
        }
    }

    // --- single fields -------------------------------------------------------------------------

    #[test]
    fn screen_name_limit_is_inclusive() {
        assert!(validate_screen_name(&"a".repeat(MAX_NAME_CHARS)).is_ok());
        let err = validate_screen_name(&"a".repeat(MAX_NAME_CHARS + 1)).unwrap_err();
        assert_eq!(err.field, "screen_name");
        assert_eq!(err.message, SCREEN_NAME_TOO_LONG_ERROR);
    }

    #[test]
    fn screen_name_may_be_blank() {
        assert!(validate_screen_name("").is_ok());
        assert!(validate_screen_name("   ").is_ok());
    }

    #[test]
    fn limits_count_chars_not_bytes() {
        // "é" is 2 bytes and "🐸" is 4, so these would fail a `.len()` check.
        assert!(validate_screen_name(&"é".repeat(MAX_NAME_CHARS)).is_ok());
        assert!(validate_thread_title(&"🐸".repeat(MAX_TITLE_CHARS)).is_ok());
        assert!(validate_post_body(&"🐸".repeat(MAX_BODY_CHARS)).is_ok());
        assert!(validate_thread_title(&"🐸".repeat(MAX_TITLE_CHARS + 1)).is_err());
    }

    #[test]
    fn limits_ignore_surrounding_whitespace() {
        let padded = format!("  {}\n", "a".repeat(MAX_NAME_CHARS));
        assert!(validate_screen_name(&padded).is_ok());
        let padded = format!("\t{}  ", "a".repeat(MAX_TITLE_CHARS));
        assert!(validate_thread_title(&padded).is_ok());
    }

    #[test]
    fn body_limit_is_inclusive() {
        assert!(validate_post_body(&"a".repeat(MAX_BODY_CHARS)).is_ok());
        let err = validate_post_body(&"a".repeat(MAX_BODY_CHARS + 1)).unwrap_err();
        assert_eq!(err.field, "body");
        assert_eq!(err.message, POST_TOO_LONG_ERROR);
    }

    #[test]
    fn blank_body_is_empty_not_too_long() {
        for body in ["", "   ", "\n\t\n", &" ".repeat(MAX_BODY_CHARS + 1)] {
            let err = validate_post_body(body).unwrap_err();
            assert_eq!(err.message, POST_EMPTY_ERROR, "body {body:?}");
        }
    }

    #[test]
    fn title_limit_is_inclusive() {
        assert!(validate_thread_title(&"a".repeat(MAX_TITLE_CHARS)).is_ok());
        let err = validate_thread_title(&"a".repeat(MAX_TITLE_CHARS + 1)).unwrap_err();
        assert_eq!(err.field, "title");
        assert_eq!(err.message, TITLE_TOO_LONG_ERROR);
    }

    #[test]
    fn blank_title_is_empty() {
        for title in ["", "   ", "\n"] {
            let err = validate_thread_title(title).unwrap_err();
            assert_eq!(err.message, TITLE_EMPTY_ERROR, "title {title:?}");
        }
    }

    // --- whole replies -------------------------------------------------------------------------

    #[test]
    fn valid_reply_is_trimmed() {
        let post = validate_reply_fields(7, &reply("  anon  ", "\n  hello\n  world \n")).unwrap();
        assert_eq!(post.thread_id, 7);
        assert_eq!(post.screen_name.as_deref(), Some("anon"));
        // Only the ends are trimmed; inner whitespace and newlines are kept.
        assert_eq!(post.body, "hello\n  world");
    }

    #[test]
    fn blank_reply_name_becomes_none() {
        for name in ["", "   "] {
            let post = validate_reply_fields(1, &reply(name, "hi")).unwrap();
            assert_eq!(post.screen_name, None, "name {name:?}");
        }
    }

    #[test]
    fn reply_reports_every_bad_field() {
        let errors =
            validate_reply_fields(1, &reply(&"a".repeat(MAX_NAME_CHARS + 1), " ")).unwrap_err();
        assert_eq!(failed_fields(&errors), ["screen_name", "body"]);
    }

    // --- whole threads -------------------------------------------------------------------------

    #[test]
    fn valid_thread_is_trimmed() {
        let new =
            validate_thread_fields("main", &thread(" Hi there ", " anon ", " first! ")).unwrap();
        assert_eq!(new.board_slug, "main");
        assert_eq!(new.title, "Hi there");
        assert_eq!(new.screen_name.as_deref(), Some("anon"));
        assert_eq!(new.body, "first!");
    }

    #[test]
    fn blank_thread_name_becomes_none() {
        let new = validate_thread_fields("main", &thread("t", "  ", "b")).unwrap();
        assert_eq!(new.screen_name, None);
    }

    #[test]
    fn thread_reports_every_bad_field() {
        let errors =
            validate_thread_fields("main", &thread("", &"a".repeat(MAX_NAME_CHARS + 1), ""))
                .unwrap_err();
        assert_eq!(failed_fields(&errors), ["screen_name", "title", "body"]);
    }

    #[test]
    fn thread_with_only_a_bad_title_reports_only_title() {
        let errors = validate_thread_fields("main", &thread("   ", "anon", "body")).unwrap_err();
        assert_eq!(failed_fields(&errors), ["title"]);
    }
}
