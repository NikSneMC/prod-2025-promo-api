use actix_web::{
    post,
    web::{Data, Json},
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::AuthenticationError,
    database::{models::DBUser, redis::RedisPool},
    models::{Token, TokenType},
    routes::ApiError,
    util::validate::validation_errors_to_string,
};

#[derive(Deserialize, Validate)]
struct SignInRequest {
    #[validate(email, length(min = 6, max = 120))]
    email: String,

    #[validate(length(min = 8, max = 60))]
    password: String,
}

#[post("sign-in")]
pub async fn post_handler(
    pool: Data<PgPool>,
    cache: Data<RedisPool>,
    body: Json<SignInRequest>,
) -> Result<Json<SignInResponse>, ApiError> {
    body.validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    let user = if let Ok(Some(user)) = DBUser::get_by_email(&body.email, &**pool).await {
        user
    } else {
        return Err(AuthenticationError::InvalidCredentials)?;
    };

    let hasher = Argon2::default();
    hasher
        .verify_password(
            body.password.as_bytes(),
            &PasswordHash::new(&user.password_hash)?,
        )
        .map_err(|_| AuthenticationError::InvalidCredentials)?;

    let mut cache = cache.connect().await?;

    let token = Token::new(TokenType::User, user.id)
        .into_db()
        .insert(&mut cache)
        .await?
        .into_model()
        .to_string();

    Ok(Json(SignInResponse { token }))
}

#[derive(Serialize)]
struct SignInResponse {
    token: String,
}
