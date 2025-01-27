use actix_web::{
    get,
    web::{Data, Path, Query},
    HttpResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    database::models::DBComment,
    models::{Comment, PromoPath},
    routes::ApiError,
};

#[derive(Deserialize, Validate)]
struct GetCommentsQuery {
    #[validate(range(min = 0))]
    limit: Option<u32>,

    #[validate(range(min = 0))]
    offset: Option<u32>,
}

#[get("")]
pub async fn get_handler(
    pool: Data<PgPool>,
    path: Path<PromoPath>,
    query: Query<GetCommentsQuery>,
) -> Result<HttpResponse, ApiError> {
    let (comments, count) = DBComment::get_pageable(
        path.promo_id,
        query.limit.unwrap_or(10).into(),
        query.offset.unwrap_or(0).into(),
        &**pool,
    )
    .await?;

    let comments: Vec<Comment> = comments.into_iter().map(DBComment::into_model).collect();

    Ok(HttpResponse::Ok()
        .insert_header(("X-Total-Count", count))
        .json(comments))
}
