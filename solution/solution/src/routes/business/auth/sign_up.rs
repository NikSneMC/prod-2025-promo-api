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
    database::{models::DBCompany, redis::RedisPool},
    models::{Company, Token, TokenType},
    routes::ApiError,
    util::validate::{validate_password, validation_errors_to_string},
};

#[derive(Deserialize, Validate, Debug)]
struct SignUpRequest {
    #[validate(length(min = 5, max = 50))]
    name: String,

    #[validate(email, length(min = 8, max = 120))]
    email: String,

    #[validate(custom(function = "validate_password"), length(min = 8, max = 256))]
    password: String,
}

#[post("sign-up")]
pub async fn post_handler(
    pool: Data<PgPool>,
    cache: Data<RedisPool>,
    body: Json<SignUpRequest>,
) -> Result<Json<SignUpResponse>, ApiError> {
    body.validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    if DBCompany::get_by_email(&body.email, &**pool)
        .await?
        .is_some()
    {
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

    let token = Token::new(TokenType::Company, id)
        .into_db()
        .insert(&mut cache)
        .await?
        .into_model()
        .to_string();

    let company = Company {
        id,
        name: body.name.clone(),
        email: body.email.clone(),
        password_hash,
    }
    .into_db()
    .insert(&mut transaction)
    .await?
    .into_model();

    transaction.commit().await?;

    Ok(Json(SignUpResponse {
        token,
        company_id: company.id.to_string(),
    }))
}

#[derive(Serialize, Debug)]
struct SignUpResponse {
    token: String,
    company_id: String,
}
