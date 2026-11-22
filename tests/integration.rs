/// Integration tests. Presently just a set of tests for the routes.
use warp::Reply;

mod common;

#[tokio::test]
async fn test_healthz() {
    let pool = common::setup().await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request().path("/healthz").filter(&filter).await;
    assert!(resp.is_ok());
    let reply = resp.unwrap().into_response();
    assert_eq!(reply.status(), 200);
}

#[tokio::test]
async fn test_get_root_returns_200() {
    let pool = common::setup().await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .path("/")
        .filter(&filter)
        .await
        .unwrap()
        .into_response();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_get_boards_with_nonexistent_board_returns_404() {
    let pool = common::setup().await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .path("/nothing_exists_since_the_db_is_empty")
        .filter(&filter)
        .await
        .unwrap()
        .into_response();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_boards_with_real_board_returns_200() {
    let pool = common::setup().await;
    common::create_board(&pool).await;
    let filter = bimage_boarb::routes(pool);
    let resp = warp::test::request()
        .path("/main")
        .filter(&filter)
        .await
        .unwrap()
        .into_response();
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
        .filter(&filter)
        .await
        .unwrap()
        .into_response();
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
        .filter(&filter)
        .await
        .unwrap()
        .into_response();
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
        .filter(&filter)
        .await
        .unwrap()
        .into_response();
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
        .filter(&filter)
        .await
        .unwrap()
        .into_response();
    assert_eq!(resp.status(), 303);
}
