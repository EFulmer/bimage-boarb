pub enum RouteError {
    BoardNotFound(String),
    ThreadNotFound(i64),
    Db(sqlx::Error),
}
