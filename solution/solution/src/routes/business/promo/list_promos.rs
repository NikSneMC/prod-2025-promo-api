use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use actix_web_lab::extract::Query;
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    models::{Promo, SortPromosBy, Token},
    routes::ApiError,
    util::validate::validate_countries,
};

#[derive(Deserialize, Validate)]
struct ListPromosQuery {
    #[validate(range(min = 0))]
    limit: Option<u32>,

    #[validate(range(min = 0))]
    offset: Option<u32>,

    sort_by: Option<SortPromosBy>,

    #[validate(custom(function = "validate_countries"))]
    country: Option<Vec<String>>,
}

#[get("")]
pub async fn get_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    query: Query<ListPromosQuery>,
) -> Result<HttpResponse, ApiError> {
    let (promos, count) = Promo::get_pageable(
        token.entity,
        query.limit,
        query.offset,
        &query.sort_by,
        &query.country,
        &**pool,
    )
    .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("X-Total-Count", count))
        .json(promos))
}
