use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let main = warp::path!("boards" / "main").map(|| "This is the main board. Big things will be happening here!");
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
