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
    database::{models::DBCompany, redis::RedisPool},
    models::{Token, TokenType},
    routes::ApiError,
    util::validate::validation_errors_to_string,
};

#[derive(Deserialize, Validate, Debug)]
struct SignInRequest {
    #[validate(email, length(min = 8, max = 120))]
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

    let company = if let Ok(Some(company)) = DBCompany::get_by_email(&body.email, &**pool).await {
        company
    } else {
        return Err(AuthenticationError::InvalidCredentials)?;
    };

    let hasher = Argon2::default();
    hasher
        .verify_password(
            body.password.as_bytes(),
            &PasswordHash::new(&company.password_hash)?,
        )
        .map_err(|_| AuthenticationError::InvalidCredentials)?;

    let mut cache = cache.connect().await?;

    let token = Token::new(TokenType::Company, company.id)
        .into_db()
        .insert(&mut cache)
        .await?
        .into_model()
        .to_string();

    Ok(Json(SignInResponse { token }))
}

#[derive(Serialize, Debug)]
struct SignInResponse {
    token: String,
}
