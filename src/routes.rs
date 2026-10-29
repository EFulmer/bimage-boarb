use maud::{Markup, html};
use sqlx::SqlitePool;
use warp::{Reply, http::StatusCode, reply::Html, reply::Response};

use crate::{db, models};

pub fn healthz() -> impl warp::Reply {
    let resp_body = models::Healthz { ok: true };
    warp::reply::with_status(warp::reply::json(&resp_body), StatusCode::OK)
}

pub async fn boards(pool: SqlitePool) -> Response {
    match db::all_boards(&pool).await {
        Ok(boards) => {
            let markup = html! {
                h1 { "All Boards" }
                @for board in boards { (render_board(&board)) }
            };
            warp::reply::html(markup.into_string()).into_response()
        }
        Err(e) => {
            eprintln!("all_boards failed: {e}");
            warp::reply::with_status("Internal server error", StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        }
    }
}

pub async fn board(pool: SqlitePool, board_id: i64) -> Response {
    // TODO need to handle an inactive board.
    match db::board_by_id(&pool, board_id).await {
        Ok(board) => {
            // TODO need to handle when there ain't no threads
            let threads = db::threads_by_board(&pool, board_id)
                .await
                .unwrap_or(Vec::new());
            let markup = html! {
                h1 { (board.name) }
                h2 { (board.description) }
                @for thread in threads {
                    h3 { (thread.title) "-" (thread.created_at) }
                }
            };
            warp::reply::html(markup.into_string()).into_response()
        }
        Err(e) => {
            eprintln!("board_by_id failed: {e}");
            warp::reply::with_status("Not Found", StatusCode::NOT_FOUND).into_response()
        }
    }
}

pub fn thread() -> Html<String> {
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
        board: 0,
        created_at: 0,
        bumped_at: 0,
        active: true,
        image_id: None,
        body: String::from("This is my board now."),
        title: String::from("First thread ever?"),
    }]
}

fn board_template(name: &str, description: &str) -> Html<String> {
    let markup = html! {
        h1 { (name) " - " (description) }
    };
    warp::reply::html(markup.into_string())
}

fn render_board(board: &models::Board) -> Markup {
    html! {
        div.board id=(board.id) {
            span.board { (board.name) "-" (board.description) }
        }
    }
}
