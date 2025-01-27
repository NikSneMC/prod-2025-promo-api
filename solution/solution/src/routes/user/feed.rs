use actix_web::{
    get,
    middleware::from_fn,
    web::{scope, Data, Query, ReqData, ServiceConfig},
    HttpResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::auth_middleware_usr,
    models::{Token, UserPromo},
    routes::ApiError,
    util::validate::validation_errors_to_string,
};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("feed")
            .wrap(from_fn(auth_middleware_usr))
            .service(get_handler),
    );
}

#[derive(Deserialize, Validate)]
struct PromosFeedQuery {
    #[validate(range(min = 0))]
    limit: Option<u32>,

    #[validate(range(min = 0))]
    offset: Option<u32>,

    #[validate(length(min = 2, max = 20))]
    category: Option<String>,

    active: Option<bool>,
}

#[get("")]
async fn get_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    query: Query<PromosFeedQuery>,
) -> Result<HttpResponse, ApiError> {
    query
        .validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    let user = token.get_user(&**pool).await?;

    let (promos, count) = UserPromo::get_pageable(
        &user,
        query.limit,
        query.offset,
        query.category.as_deref(),
        query.active,
        &**pool,
    )
    .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("X-Total-Count", count))
        .json(promos))
}
