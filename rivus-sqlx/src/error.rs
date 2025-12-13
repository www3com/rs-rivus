use std::fmt;

#[derive(Debug)]
pub enum DbError {
    Sqlx(sqlx::Error),
    Config(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Sqlx(e) => write!(f, "Database error: {}", e),
            DbError::Config(e) => write!(f, "Configuration error: {}", e),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DbError::Sqlx(e) => Some(e),
            DbError::Config(_) => None,
        }
    }
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        DbError::Sqlx(err)
    }
}

impl From<String> for DbError {
    fn from(err: String) -> Self {
        DbError::Config(err)
    }
}

impl From<&str> for DbError {
    fn from(err: &str) -> Self {
        DbError::Config(err.to_string())
    }
}
