use actix_web::{
    post,
    web::{Data, Json},
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthenticationError,
    database::{models::DBUser, redis::RedisPool},
    models::{Token, TokenType, User, UserTargetSettings},
    routes::ApiError,
    util::validate::{validate_password, validation_errors_to_string},
};

#[derive(Deserialize, Validate, Clone, Debug)]
pub struct SignUpRequest {
    #[validate(length(min = 1, max = 100))]
    name: String,

    #[validate(length(min = 1, max = 120))]
    surname: String,

    #[validate(email, length(min = 6, max = 120))]
    email: String,

    #[validate(url, length(max = 350))]
    avatar_url: Option<String>,

    #[validate(nested)]
    other: UserTargetSettings,

    #[validate(custom(function = "validate_password"), length(min = 8, max = 256))]
    password: String,
}

#[post("sign-up")]
async fn post_handler(
    pool: Data<PgPool>,
    cache: Data<RedisPool>,
    body: Json<SignUpRequest>,
) -> Result<Json<SignUpResponse>, ApiError> {
    body.validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    if DBUser::get_by_email(&body.email, &**pool).await?.is_some() {
        return Err(AuthenticationError::DuplicateCompany)?;
    }

    let id = Uuid::now_v7();
    let hasher = Argon2::default();
    let salt = SaltString::generate(&mut ChaCha20Rng::from_entropy());
    let password_hash = hasher
        .hash_password(body.password.as_bytes(), &salt)?
        .to_string();

    let mut transaction = pool.begin().await?;
    let mut cache = cache.connect().await?;

    let token = Token::new(TokenType::User, id)
        .into_db()
        .insert(&mut cache)
        .await?
        .into_model()
        .to_string();

    User {
        id,
        name: body.name.clone(),
        surname: body.surname.clone(),
        email: body.email.clone(),
        avatar_url: body.avatar_url.clone(),
        other: body.other.clone(),
        password_hash,
    }
    .into_db()
    .insert(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(Json(SignUpResponse { token }))
}

#[derive(Serialize, Debug)]
struct SignUpResponse {
    token: String,
}
