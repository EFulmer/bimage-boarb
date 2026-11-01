use sqlx::{Error, SqlitePool};
use warp::{Reply, http::StatusCode, reply::Response};

use crate::{models, queries, templates};

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
    // TODO: actual params, looking up the thread in da database, etc.
    match queries::board_by_slug(&pool, &board_slug).await {
        Ok(board) => match queries::thread_by_id(&pool, board.id, thread_id).await {
            Ok(thread) => {
                println!("still a WIP lol");
                let my_posts = posts();
                warp::reply::html(templates::thread_page(&board, &thread, &my_posts).into_string())
                    .into_response()
            }
            Err(Error::RowNotFound) => {
                eprintln!("board not found: {board_slug}");
                not_found_page(
                    "Thread",
                    &thread_id.to_string(),
                    "This thread is unavailable... \
                        or maybe it was never available in the first place? Much to think about.",
                )
            }
            Err(e) => {
                eprintln!("thread_by_id failed: {e}");
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

fn posts() -> Vec<models::Post> {
    vec![
        models::Post {
            id: 1,
            thread_id: 0,
            created_at: 0,
            active: true,
            image_id: None,
            body: String::from("Tope!"),
            screen_name: Some(String::from("Shane")),
            in_reply_to: None,
        },
        models::Post {
            thread_id: 0,
            id: 2,
            created_at: 1000,
            active: true,
            image_id: None,
            screen_name: Some(String::from("Eric")),
            body: String::from("Tup top nory."),
            in_reply_to: Some(1),
        },
        models::Post {
            id: 3,
            thread_id: 0,
            created_at: 20000,
            active: false,
            image_id: None,
            screen_name: Some(String::from("Stef")),
            body: String::from("D'oh!"),
            in_reply_to: Some(1),
        },
    ]
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
