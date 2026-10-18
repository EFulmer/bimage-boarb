use maud::{html, Markup};
use serde;
use warp::{Filter, http::StatusCode, reply::Html};

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let main = warp::path!("boards" / "main").map(|| board_route("main", "main"));
    // TODO: URL param for thread ID
    let thread = warp::path!("boards" / "main" / "threads" / "0").map(|| thread_route());
    let admin = warp::path("admin").map(|| "This is the admin view. Most people aren't going to have access to what lives in these routes, which shouldn't be a surprise.");
    let healthz = warp::path("healthz").map(|| healthz_route());

    let all_routes = warp::get().and(
        hello
        .or(main)
        .or(thread)
        .or(admin)
        .or(healthz),
    );

    warp::serve(all_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

// Keeping everything one file for now.
// Routes begin below:
fn healthz_route() -> impl warp::Reply {
    let resp_body = Healthz{ok: true};
    warp::reply::with_status(warp::reply::json(&resp_body), StatusCode::OK)
}

fn board_route(name: &str, description: &str) -> Html<String> {
    // TODO: check the board exists, query da db, etc.
    board_template(name, description)
}

fn thread_route() -> Html<String> {
    // TODO: actual params, looking up the thread in da database, etc.
    let my_threads = threads();
    thread_template(&my_threads[0])
}

// Routes end above.

// Renderers begin below:
fn board_template(name: &str, description: &str) -> Html<String> {
    let markup = html! {
        h1 { (name) " - " (description) }
    };
    warp::reply::html(markup.into_string())
}

fn thread_template(thread: &Thread) ->  Html<String> {
    // Obviously this `let` immediately below is to populate with some fake data to test rendering.
    let my_posts = posts();
    let markup = html! {
        h2 { (thread.title) }
        @for p in my_posts { (post(&p)) }
    };
    warp::reply::html(markup.into_string())
}

fn post(post: &Post) -> Markup {
    html! {
        div.post id=(post.id) {
            span.screen_name { (post.screen_name.clone().unwrap_or("Zorro".to_string())) }
            (post.body)
        }
    }
}

// Renderers end above.

// Types begin below:
#[derive(serde::Serialize)]
struct Healthz {
    ok: bool,
}

#[derive(sqlx::FromRow)]
struct Board {
    id: i64,
    name: String,
    slug: String,
    description: String,
    active: bool,
}

#[derive(sqlx::FromRow)]
struct Thread {
    id: i64,
    board: i64,
    created_at: i64,
    bumped_at: i64,
    active: bool,
    image_id: Option<i64>,
    title: String,
    body: String,
}

#[derive(sqlx::FromRow)]
struct Post {
    id: i64,
    thread_id: i64,
    created_at: i64,
    active: bool,
    image_id: Option<i64>,
    screen_name: Option<String>,
    body: String,
    in_reply_to: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct Image {
    id: i64,
    uri: String,
    sha256: String,
    mime_type: String,
    original_name: Option<String>,
}

// Types end above.

// Generic data helpers for testing stuff begin below.
fn posts() -> Vec<Post> {
    vec![
        Post{
            id: 1,
            thread_id: 0,
            created_at: 0,
            active: true,
            image_id: None,
            body: String::from("Tope!"),
            screen_name: Some(String::from("Shane")),
            in_reply_to: None,
        },
        Post{
            thread_id: 0,
            id: 2,
            created_at: 1000,
            active: true,
            image_id: None,
            screen_name: Some(String::from("Eric")),
            body: String::from("Tup top nory."),
            in_reply_to: Some(1),
        },
        Post{
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

fn threads() -> Vec<Thread> {
    vec![
        Thread{
            id: 0,
            board: 0,
            created_at: 0,
            bumped_at: 0,
            active: true,
            image_id: None,
            body: String::from("This is my board now."),
            title: String::from("First thread ever?"),
        }
    ]
}

// Generic data helpers end above.
