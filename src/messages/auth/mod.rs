use crate::{
    errors::auth::{LoginError, RegisterError},
    models::user::User,
};
use actix::Message;

#[derive(Message)]
#[rtype(result = "Result<User,RegisterError>")]
pub struct RegisterMessage {
    pub username: String,
    pub password: String,
}

#[derive(Message)]
#[rtype(result = "Result<User,LoginError>")]
pub struct LoginMessage {
    pub username: String,
    pub password: String,
}
