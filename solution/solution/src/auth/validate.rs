use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::{HeaderValue, AUTHORIZATION},
    middleware::Next,
    web::Data,
    Error, HttpMessage,
};

use crate::{
    database::redis::RedisPool,
    models::{Token, TokenType},
    routes::ApiError,
};

use super::AuthenticationError;

pub async fn auth_middleware_cmp(
    cache: Data<RedisPool>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    auth_middleware(cache, req, next, TokenType::Company).await
}

pub async fn auth_middleware_usr(
    cache: Data<RedisPool>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    auth_middleware(cache, req, next, TokenType::User).await
}

pub async fn auth_middleware(
    cache: Data<RedisPool>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
    allowed_tokens: TokenType,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let mut cache = cache.connect().await.map_err(ApiError::Database)?;

    let token = extract_token_from_authorization_header(&req)?
        .validate(&mut cache)
        .await?;

    if token.token_type != allowed_tokens {
        return Err(AuthenticationError::IcorrectTokenType)?;
    }

    req.extensions_mut().insert(token);

    next.call(req).await
}

pub fn extract_token_from_authorization_header(
    req: &ServiceRequest,
) -> Result<Token, AuthenticationError> {
    let headers = req.headers();
    let token_val: Option<&HeaderValue> = headers.get(AUTHORIZATION);
    let token_val = token_val
        .ok_or_else(|| AuthenticationError::NoAuthorizationHeader)?
        .to_str()
        .map_err(|_| AuthenticationError::InvalidCredentials)?;
    if let Some(token) = token_val.strip_prefix("Bearer ") {
        Ok(Token::from_str(token).map_err(|_| AuthenticationError::InvalidCredentials)?)
    } else {
        Err(AuthenticationError::InvalidAuthMethod)
    }
}
