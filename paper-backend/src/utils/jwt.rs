use jsonwebtoken::{EncodingKey, Header, decode, encode};
use serde::{Deserialize, Serialize};

use std::sync::OnceLock;

use crate::error::ServiceResult;
static JWT_SECRET: OnceLock<String> = OnceLock::new();

pub fn set_jwt_secret(secret: String) {
    JWT_SECRET.set(secret).ok();
}

pub fn get_jwt_secret() -> &'static str {
    JWT_SECRET
        .get()
        .map(|s| s.as_str())
        .expect("JWT secret not set")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    // (subject): Subject of the JWT (the user)
    pub sub: String,
    // (issued at time): Time at which the JWT was issued;
    // can be used to determine age of the JWT.
    pub iat: i64,
    // (expiration time): Time after which the JWT expires
    pub exp: i64,
}

impl JwtClaims {
    pub fn new(sub: String, iat: i64, exp: i64) -> Self {
        JwtClaims { sub, iat, exp }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() + 3580 > self.exp
    }

    pub fn encode(&self) -> ServiceResult<String> {
        Ok(encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
        )?)
    }

    pub fn decode(token: &str) -> ServiceResult<Self> {
        let decoded = decode::<JwtClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(get_jwt_secret().as_bytes()),
            &jsonwebtoken::Validation::default(),
        )?;
        Ok(decoded.claims)
    }
}

pub fn generate_jwt_token(sub: String) -> ServiceResult<String> {
    let current_time = chrono::Utc::now().timestamp();
    let expiration_time = current_time + 3600; // 1 hour expiration
    let claims = JwtClaims::new(sub, current_time, expiration_time);
    claims.encode()
}
