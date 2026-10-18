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
struct Post {
    active: bool,
    body: String,
    id: u64,
    image: Option<u64>,
    in_reply_to: Option<u64>,
    thread_id: u64,
    screen_name: Option<String>,
}

struct Thread {
    active: bool,
    board: u64,
    body: String,
    id: u64,
    image: Option<u64>,
    title: String,
}

// Types end above.

// Generic data helpers for testing stuff begin below.
fn posts() -> Vec<Post> {
    vec![
        Post{
            active: true,
            body: String::from("Tope!"),
            id: 1,
            image: None,
            in_reply_to: None,
            thread_id: 0,
            screen_name: Some(String::from("Shane")),
        },
        Post{
            active: true,
            body: String::from("Tup top nory."),
            id: 2,
            image: None,
            in_reply_to: Some(1),
            thread_id: 0,
            screen_name: Some(String::from("Eric")),
        },
        Post{
            active: false,
            body: String::from("D'oh!"),
            id: 3,
            image: None,
            in_reply_to: Some(1),
            thread_id: 0,
            screen_name: Some(String::from("Stef")),
        },
    ]
}

fn threads() -> Vec<Thread> {
    vec![
        Thread{
            active: true,
            board: 0,
            body: String::from("This is my board now."),
            id: 0,
            image: None,
            title: String::from("First thread ever?"),
        }
    ]
}

// Generic data helpers end above.
