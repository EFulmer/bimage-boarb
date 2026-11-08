use std::convert::Infallible;

use sqlx::{Error, SqlitePool};
use warp::{
    Rejection, Reply, http::StatusCode, path::FullPath, reject::MethodNotAllowed, reply::Response,
};

use crate::{errors, models, queries, templates};

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
    match get_board(&pool, &board_slug).await {
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
    match get_thread(&pool, &board_slug, thread_id).await {
        Ok(thread_template) => warp::reply::html(thread_template.into_string()).into_response(),
        Err(errors::ThreadRouteError::BoardNotFound(board_slug)) => {
            tracing::warn!("board not present or inactive in database");
            board_not_found(&board_slug)
        }
        Err(errors::ThreadRouteError::ThreadNotFound(thread_id)) => {
            tracing::warn!("thread not present or inactive in database");
            not_found_page(
                "Thread",
                &thread_id.to_string(),
                "This thread is unavailable... \
                        or maybe it was never available in the first place? Much to think about.",
            )
        }
        Err(errors::ThreadRouteError::Db(error)) => {
            tracing::error!(?error, "query for thread or board failed");
            internal_server_error_page()
        }
    }
}

async fn get_board(
    pool: &SqlitePool,
    board_slug: &str,
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
    Ok(templates::board_page(&board, &threads))
}

async fn get_thread(
    pool: &SqlitePool,
    board_slug: &str,
    thread_id: i64,
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
    Ok(templates::thread_page(
        &board,
        &thread,
        &posts,
        &templates::ReplyForm::default(),
    ))
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
    if err.find::<MethodNotAllowed>().is_some() {
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
