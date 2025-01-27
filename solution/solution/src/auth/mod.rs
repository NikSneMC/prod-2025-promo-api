use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

use crate::models::ApiError;

mod validate;

pub use validate::{auth_middleware, auth_middleware_cmp, auth_middleware_usr};

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("An unknown database error occurred: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),

    #[error("Error while parsing JSON: {0}")]
    SerDe(#[from] serde_json::Error),

    #[error("Missing Authorization Header")]
    NoAuthorizationHeader,

    #[error("Invalid Authentication Credentials")]
    InvalidCredentials,

    #[error("Authentication method was not valid")]
    InvalidAuthMethod,

    #[error("Incorrect token type")]
    IcorrectTokenType,

    #[error("Company email/account is already registered")]
    DuplicateCompany,

    #[error("User email/account is already registered")]
    DuplicateUser,
}

impl ResponseError for AuthenticationError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Sqlx(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SerDe(..) => StatusCode::BAD_REQUEST,
            Self::NoAuthorizationHeader => StatusCode::UNAUTHORIZED,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::InvalidAuthMethod => StatusCode::UNAUTHORIZED,
            Self::IcorrectTokenType => StatusCode::FORBIDDEN,
            Self::DuplicateCompany => StatusCode::CONFLICT,
            Self::DuplicateUser => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: self.error_name(),
            description: self.to_string(),
        })
    }
}

impl AuthenticationError {
    pub fn error_name(&self) -> &'static str {
        match self {
            Self::Sqlx(..) => "database_error",
            Self::Database(..) => "database_error",
            Self::SerDe(..) => "invalid_input",
            Self::NoAuthorizationHeader => "missing_authorization_header",
            Self::InvalidCredentials => "invalid_credentials",
            Self::InvalidAuthMethod => "invalid_auth_method",
            Self::IcorrectTokenType => "incorrect_token_type",
            Self::DuplicateCompany => "duplicate_company",
            Self::DuplicateUser => "duplicate_user",
        }
    }
}
