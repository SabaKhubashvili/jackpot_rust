use crate::errors::auth::LoginError;
use crate::messages::auth::LoginMessage;
use crate::schema::users::dsl::{username as user_username, users};
use crate::{
    db_utils::DbActor,
    errors::auth::RegisterError,
    messages::auth::RegisterMessage,
    models::user::{NewUser, User},
    validation::{validate_generic, validate_username},
};
use actix::Handler;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::Error;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{PgConnection, RunQueryDsl};
pub fn username_available(
    username: &str,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> bool {
    match users.filter(user_username.eq(username)).first::<User>(conn) {
        Ok(_) => false, // Username exists
        Err(Error::NotFound) => true,
        Err(_) => false, // Other errors, treat as username not available
    }
}

impl Handler<RegisterMessage> for DbActor {
    type Result = Result<User, RegisterError>;
    fn handle(&mut self, msg: RegisterMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.0.get().expect("Failed to get connection");
        if !validate_username(&msg.username) || !validate_generic(&msg.password) {
            return Err(RegisterError::ForbiddenFormat);
        }
        if !username_available(&msg.username, &mut conn) {
            return Err(RegisterError::UsernameAlreadyRegistered);
        }
        let password_hash: String =
            hash(&msg.password, DEFAULT_COST).map_err(|_| RegisterError::InternalError)?;
        let new_user = NewUser {
            username: msg.username,
            hashed_password: password_hash,
            balance: 0,
        };

        diesel::insert_into(users)
            .values(&new_user)
            .get_result::<User>(&mut conn)
            .map_err(|e| RegisterError::DieselError(e))
    }
}

impl Handler<LoginMessage> for DbActor {
    type Result = Result<User, LoginError>;

    fn handle(&mut self, msg: LoginMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.0.get().expect("Failed to get connection");

        match users
            .filter(user_username.eq(msg.username))
            .first::<User>(&mut conn)
        {
            Ok(user) => {
                let user_hashed_pass = &user.hashed_password;
                if verify(&msg.password, user_hashed_pass).map_err(|_| LoginError::InternalError)? {
                    Ok(user)
                } else {
                    Err(LoginError::InvalidCredentials)
                }
            }
            Err(Error::NotFound) => Err(LoginError::InvalidCredentials),
            Err(_) => Err(LoginError::InternalError),
        }
    }
}
