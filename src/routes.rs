use sqlx::{Error, SqlitePool};
use warp::{Reply, http::StatusCode, reply::Response};

use crate::{models, queries, templates, types};

pub fn healthz() -> impl warp::Reply {
    let resp_body = models::Healthz { ok: true };
    warp::reply::with_status(warp::reply::json(&resp_body), StatusCode::OK)
}

pub async fn boards(pool: SqlitePool) -> Response {
    match queries::all_boards(&pool).await {
        Ok(boards) => {
            warp::reply::html(templates::boards_page(&boards).into_string()).into_response()
        }
        Err(e) => {
            eprintln!("all_boards failed: {e}");
            internal_server_error_page()
        }
    }
}

pub async fn board(pool: SqlitePool, board_slug: String) -> Response {
    match queries::board_by_slug(&pool, &board_slug).await {
        Ok(board) => match queries::threads_by_board(&pool, board.id).await {
            Ok(threads) => warp::reply::html(templates::board_page(&board, &threads).into_string())
                .into_response(),
            Err(e) => {
                eprintln!("error when querying database for threads on board {board_slug}: {e}");
                internal_server_error_page()
            }
        },
        Err(Error::RowNotFound) => {
            eprintln!("board not found: {board_slug}");
            board_not_found(&board_slug)
        }
        Err(e) => {
            eprintln!("board_by_slug failed: {e}");
            internal_server_error_page()
        }
    }
}

pub async fn thread(pool: SqlitePool, board_slug: String, thread_id: i64) -> Response {
    match get_thread(&pool, &board_slug, thread_id).await {
        Ok(thread_template) => warp::reply::html(thread_template.into_string()).into_response(),
        Err(types::RouteError::BoardNotFound(bs)) => {
            eprintln!("board not found: {bs}");
            board_not_found(&bs)
        }
        Err(types::RouteError::ThreadNotFound(tid)) => {
            eprintln!("thread not found! {board_slug}/threads/{tid}");
            not_found_page(
                "Thread",
                &thread_id.to_string(),
                "This thread is unavailable... \
                        or maybe it was never available in the first place? Much to think about.",
            )
        }
        Err(types::RouteError::Db(e)) => {
            eprintln!("query failed: {e}");
            internal_server_error_page()
        }
    }
}

async fn get_thread(
    pool: &SqlitePool,
    board_slug: &str,
    thread_id: i64,
) -> Result<maud::Markup, types::RouteError> {
    let board = queries::board_by_slug(pool, board_slug)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => types::RouteError::BoardNotFound(board_slug.to_string()),
            e => types::RouteError::Db(e),
        })?;
    let thread = queries::thread_by_id(pool, board.id, thread_id)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => types::RouteError::ThreadNotFound(thread_id),
            e => types::RouteError::Db(e),
        })?;
    let posts = queries::posts_by_thread(pool, thread.id)
        .await
        .map_err(types::RouteError::Db)?;
    Ok(templates::thread_page(&board, &thread, &posts))
}

fn internal_server_error_page() -> Response {
    error_page_with_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error.",
        "The board is having issues. The specifics have been logged for an admin to fix.",
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
