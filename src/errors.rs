#[derive(Debug)]
pub enum GetBoardError {
    NotFound(String),
    Db(sqlx::Error),
}

#[derive(Debug)]
pub enum GetThreadError {
    BoardNotFound(String),
    ThreadNotFound(i64),
    Db(sqlx::Error),
}

#[derive(Debug)]
pub enum QueryError {
    NotFound(Object),
    Db(sqlx::Error),
}

impl From<sqlx::Error> for QueryError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}

#[derive(Debug)]
pub enum Object {
    Board(String),
    Thread(i64),
}
