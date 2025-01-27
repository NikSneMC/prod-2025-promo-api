use actix_web::{
    patch,
    web::{Data, Json, ReqData},
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    database::models::DBUser,
    models::{Token, UserTargetSettings},
    routes::ApiError,
    util::validate::{validate_password, validation_errors_to_string},
};

#[derive(Deserialize, Validate, Debug)]
struct EditProfileRequest {
    #[validate(length(min = 1, max = 100))]
    name: Option<String>,

    #[validate(length(min = 1, max = 120))]
    surname: Option<String>,

    #[validate(url, length(max = 350))]
    avatar_url: Option<String>,

    #[validate(custom(function = "validate_password"), length(min = 8, max = 256))]
    password: Option<String>,
}

#[patch("")]
pub async fn patch_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    body: Json<EditProfileRequest>,
) -> Result<Json<EditProfileResponse>, ApiError> {
    body.validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    let mut password_hash: Option<String> = None;

    if let Some(password) = &body.password {
        let hasher = Argon2::default();
        let salt = SaltString::generate(&mut ChaCha20Rng::from_entropy());
        password_hash = Some(
            hasher
                .hash_password(password.as_bytes(), &salt)?
                .to_string(),
        );
    }

    let mut transaction = pool.begin().await?;

    let user = DBUser::patch(
        token.entity,
        body.name.clone().as_deref(),
        body.surname.clone().as_deref(),
        body.avatar_url.clone().as_deref(),
        password_hash.clone().as_deref(),
        &mut transaction,
    )
    .await?
    .into_model();

    transaction.commit().await?;

    Ok(Json(EditProfileResponse {
        name: user.name,
        surname: user.surname,
        email: user.email,
        avatar_url: user.avatar_url,
        other: user.other,
    }))
}

#[derive(Serialize, Debug)]
struct EditProfileResponse {
    name: String,
    surname: String,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    other: UserTargetSettings,
}
