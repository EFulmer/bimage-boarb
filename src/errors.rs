pub enum BoardRouteError {
    NotFound(String),
    Db(sqlx::Error),
}

pub enum ThreadRouteError {
    BoardNotFound(String),
    ThreadNotFound(i64),
    Db(sqlx::Error),
}
