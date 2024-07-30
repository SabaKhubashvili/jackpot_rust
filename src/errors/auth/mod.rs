use diesel::result::Error as DieselError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Forbidden format")]
    ForbiddenFormat,
    #[error("Username already registered")]
    UsernameAlreadyRegistered,
    #[error("Internal error")]
    InternalError,
    #[error("Diesel error")]
    DieselError(#[from] DieselError),
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Internal error")]
    InternalError,
    #[error("Database error")]
    DatabaseError(#[from] DieselError),
}
