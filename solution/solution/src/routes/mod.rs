use actix_web::{
    http::StatusCode,
    web::{scope, ServiceConfig},
    HttpResponse, ResponseError,
};

mod business;
mod not_found;
mod ping;
mod user;

use crate::{auth::AuthenticationError, database::models::DatabaseError, util::cors::default_cors};

pub use self::not_found::not_found;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("api")
            .wrap(default_cors())
            .service(ping::get_handler)
            .configure(business::config)
            .configure(user::config),
    );
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Database Error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Database Error: {0}")]
    SqlxDatabase(#[from] sqlx::Error),

    #[error("Deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Resource not found")]
    NotFound,

    #[error("Authentication Error: {0}")]
    Authentication(#[from] AuthenticationError),

    #[error("You're not allowed to do this")]
    NotOwner,

    #[error("You're not allowed to use this promo")]
    FraudDetected,

    #[error("This promo is expired")]
    PromoExpired,

    #[error("You can't use this promo")]
    NotPromoTarget,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Error while validating input: {0}")]
    Validation(String),

    #[error("Password Hashing Error: {0}")]
    PasswordHashing(#[from] argon2::password_hash::Error),

    #[error("Error while communicating to antifraud")]
    Reqwest(#[from] reqwest::Error),
}

impl ApiError {
    pub fn as_api_error<'a>(&self) -> crate::models::ApiError<'a> {
        crate::models::ApiError {
            error: match self {
                Self::SqlxDatabase(..) => "database_error",
                Self::Database(..) => "database_error",
                Self::Authentication(err) => err.error_name(),
                Self::NotOwner => "not_owner",
                Self::FraudDetected => "fraud_suspence",
                Self::PromoExpired => "promo_expired",
                Self::NotPromoTarget => "not_promo_target",
                Self::Json(..) => "json_error",
                Self::NotFound => "not_found",
                Self::InvalidInput(..) => "invalid_input",
                Self::Validation(..) => "invalid_input",
                Self::PasswordHashing(..) => "password_hashing_error",
                Self::Reqwest(..) => "network_error",
            },
            description: self.to_string(),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SqlxDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Authentication(err) => err.status_code(),
            Self::NotOwner => StatusCode::FORBIDDEN,
            Self::FraudDetected => StatusCode::FORBIDDEN,
            Self::PromoExpired => StatusCode::FORBIDDEN,
            Self::NotPromoTarget => StatusCode::FORBIDDEN,
            Self::Json(..) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InvalidInput(..) => StatusCode::BAD_REQUEST,
            Self::Validation(..) => StatusCode::BAD_REQUEST,
            Self::PasswordHashing(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Reqwest(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}
