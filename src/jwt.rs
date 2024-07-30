use std::env;

use jsonwebtoken::{
    decode, encode, errors::Error, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub exp: usize,
    pub sub: i32,
}

pub fn generate_jwt(username: &str, id: i32, exp: usize) -> Result<String, Error> {
    let claims = Claims {
        username: username.to_string(),
        exp,
        sub: id,
    };
    let secret_key = env::var("JWT_SECRET_KEY").expect("secret key not found");
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    );
    token
}

pub fn decode_jwt(token: &str) -> Result<TokenData<Claims>, Error> {
    let secret_key = env::var("JWT_SECRET_KEY").expect("secret key not found");
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret_key.as_ref()),
        &Validation::default(),
    );
    decoded
}
