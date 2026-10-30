use maud::{Markup, html};
use sqlx::{Error, SqlitePool};
use warp::{Reply, http::StatusCode, reply::Html, reply::Response};

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

pub async fn board(pool: SqlitePool, board_id: i64) -> Response {
    match queries::board_by_id(&pool, board_id).await {
        Ok(board) => match queries::threads_by_board(&pool, board_id).await {
            Ok(threads) => warp::reply::html(templates::board_page(&board, &threads).into_string())
                .into_response(),
            Err(e) => {
                eprintln!("error when querying database for threads on board {board_id}: {e}");
                internal_server_error_page()
            }
        },
        Err(Error::RowNotFound) => {
            eprintln!("board not found: {board_id}");
            error_page_with_response(
                StatusCode::NOT_FOUND,
                &format!("Board {board_id} not found!"),
                &format!(
                    "The board {board_id} doesn't exist. Maybe it was misspelled, moved, or hasn't been created yet?"
                ),
            )
        }
        Err(e) => {
            eprintln!("board_by_id failed: {e}");
            internal_server_error_page()
        }
    }
}

fn internal_server_error_page() -> Response {
    error_page_with_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error.",
        "The board is having issues. The specifics have been logged for an admin to fix.",
    )
}

fn error_page_with_response(status: StatusCode, heading: &str, message: &str) -> Response {
    warp::reply::with_status(
        warp::reply::html(templates::error_page(status.as_u16(), heading, message).into_string()),
        status,
    )
    .into_response()
}

pub fn thread(pool: SqlitePool, board_id: i64, thread_id: i64) -> Html<String> {
    // TODO: actual params, looking up the thread in da database, etc.
    let my_threads = threads();
    thread_template(&my_threads[0])
}

fn thread_template(thread: &models::Thread) -> Html<String> {
    // Obviously this `let` immediately below is to populate with some fake data to test rendering.
    let my_posts = posts();
    let markup = html! {
        h2 { (thread.title) }
        @for p in my_posts { (post(&p)) }
    };
    warp::reply::html(markup.into_string())
}

fn post(post: &models::Post) -> Markup {
    html! {
        div.post id=(post.id) {
            span.screen_name { (post.screen_name.clone().unwrap_or("Zorro".to_string())) }
            (post.body)
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

fn threads() -> Vec<models::Thread> {
    vec![models::Thread {
        id: 0,
        board_id: 0,
        created_at: 0,
        bumped_at: 0,
        active: true,
        image_id: None,
        body: String::from("This is my board now."),
        title: String::from("First thread ever?"),
    }]
}
