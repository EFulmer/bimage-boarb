/// Integration tests. Presently just a set of tests for the routes.
mod common;

#[tokio::test]
async fn healthz() {
    let pool = common::setup().await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request().path("/healthz").reply(&filter).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn get_root_returns_200() {
    let pool = common::setup().await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request().path("/").reply(&filter).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn get_boards_with_nonexistent_board_returns_404() {
    let pool = common::setup().await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .path("/nothing_exists_since_the_db_is_empty")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn get_boards_with_real_board_returns_200() {
    let pool = common::setup().await;
    common::create_board(&pool).await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request().path("/main").reply(&filter).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn empty_form_in_make_thread_returns_400() {
    let pool = common::setup().await;
    common::create_board(&pool).await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .method("POST")
        .path("/main")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn missing_fields_in_make_thread_post_returns_400() {
    let pool = common::setup().await;
    common::create_board(&pool).await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .method("POST")
        .path("/main")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("title=loren%20ipsum")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn reply_post_to_existing_thread_with_missing_fields_returns_422() {
    let pool = common::setup().await;
    common::create_board(&pool).await;
    common::create_thread(&pool).await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .method("POST")
        .path("/main/threads/1")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("screen_name=tester&body=")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn proper_post_to_make_thread_returns_303() {
    let pool = common::setup().await;
    common::create_board(&pool).await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .method("POST")
        .path("/main")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("screen_name=johndoe&title=a-test&body=yep")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 303);
}

#[tokio::test]
async fn replying_to_nonexistent_thread_on_real_board() {
    let pool = common::setup().await;
    common::create_board(&pool).await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .method("POST")
        .path("/main/threads/555")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("screen_name=tester&body=test")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 404);
    let body = String::from_utf8_lossy(resp.body());
    assert!(body.contains("This thread is unavailable"));
}

#[tokio::test]
async fn replying_to_nonexistent_thread_on_nonexistent_board() {
    let pool = common::setup().await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .method("POST")
        .path("/nosuchboard/threads/555")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("screen_name=tester&body=test")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 404);
    let body = String::from_utf8_lossy(resp.body());
    assert!(body.contains("The board nosuchboard doesn't exist"));
}

#[tokio::test]
async fn make_thread_to_nonexistent_board_returns_404() {
    let pool = common::setup().await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .method("POST")
        .path("/nosuchboard")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("title=test%20thread&screen_name=tester&body=test")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 404);
    let body = String::from_utf8_lossy(resp.body());
    assert!(body.contains("The board nosuchboard doesn't exist"));
}

#[tokio::test]
async fn good_reply_redirects_properly() {
    let pool = common::setup().await;
    common::create_board(&pool).await;
    common::create_thread(&pool).await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .method("POST")
        .path("/main/threads/1")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("screen_name=tester&body=tope")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 303);
    assert_eq!(resp.headers()["location"], "/main/threads/1#p1");
}
