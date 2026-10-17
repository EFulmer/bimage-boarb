use maud::html;
use warp::Filter;
use warp::reply::Html;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let main = warp::path!("boards" / "main").map(|| board_template("main", "main"));
    let admin = warp::path("admin").map(|| "This is the admin view. Most people aren't going to have access to what lives in these routes, which shouldn't be a surprise.");

    let all_routes = warp::get().and(
        hello
        .or(main)
        .or(admin),
    );

    warp::serve(all_routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn board_template(name: &str, description: &str) -> Html<String> {
    let markup = html! {
        h1 { (name) " - " (description) }
    };
    warp::reply::html(markup.into_string())
}
