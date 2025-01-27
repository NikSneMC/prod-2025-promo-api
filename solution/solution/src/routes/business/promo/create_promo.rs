use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    database::models::DBPromoMode,
    models::{Promo, PromoTarget, Token},
    routes::ApiError,
    util::{convertions::promo_date_format, validate::validation_errors_to_string},
};

#[derive(Deserialize, Validate, Debug)]
struct CreatePromoRequest {
    #[validate(length(min = 10, max = 300))]
    description: String,

    #[validate(url, length(max = 350))]
    image_url: Option<String>,

    #[validate(nested)]
    target: PromoTarget,

    #[validate(range(min = 0, max = 100000000))]
    max_count: i32,

    #[serde(default, with = "promo_date_format")]
    active_from: Option<DateTime<Utc>>,

    #[serde(default, with = "promo_date_format")]
    active_until: Option<DateTime<Utc>>,

    mode: DBPromoMode,

    #[validate(length(min = 5, max = 30))]
    promo_common: Option<String>,

    #[validate(length(min = 1, max = 5000))]
    promo_unique: Option<Vec<String>>,
}

#[post("")]
pub async fn post_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    Json(body): Json<CreatePromoRequest>,
) -> Result<HttpResponse, ApiError> {
    body.validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    if body.active_from.is_some()
        && body.active_until.is_some()
        && body.active_from.unwrap() >= body.active_until.unwrap()
    {
        return Err(ApiError::InvalidInput(
            "`active_from` must be less than `active_until`".to_string(),
        ));
    }

    match body {
        CreatePromoRequest {
            mode: DBPromoMode::COMMON,
            promo_common: None,
            ..
        } => Err(ApiError::InvalidInput(
            "field `promo_common` is required for COMMON promocodes".to_string(),
        )),
        CreatePromoRequest {
            mode: DBPromoMode::UNIQUE,
            promo_unique: None,
            ..
        } => Err(ApiError::InvalidInput(
            "field `promo_unique` is required for UNIQUE promocodes".to_string(),
        )),
        CreatePromoRequest {
            mode: DBPromoMode::COMMON,
            promo_unique: Some(..),
            ..
        } => Err(ApiError::InvalidInput(
            "field `promo_unique` can't be used in COMMON promocodes".to_string(),
        )),
        CreatePromoRequest {
            mode: DBPromoMode::UNIQUE,
            promo_common: Some(..),
            ..
        } => Err(ApiError::InvalidInput(
            "field `promo_common` can't be used in UNIQUE promocodes".to_string(),
        )),
        CreatePromoRequest {
            mode: DBPromoMode::UNIQUE,
            max_count: 0 | 2..,
            ..
        } => Err(ApiError::InvalidInput(
            "field `max_count` must be 1 for UNIQUE promocodes".to_string(),
        )),
        _ => Ok(()),
    }?;

    let company = token.get_company(&**pool).await?;

    let mut transaction = pool.begin().await?;

    let promo = Promo {
        id: Uuid::now_v7(),
        company_id: company.id,
        company_name: company.name,
        description: body.description.clone(),
        image_url: body.image_url.clone(),
        target: body.target,
        max_count: body.max_count,
        active_from: body.active_from,
        active_until: body.active_until,
        mode: body.mode,
        promo_common: body.promo_common,
        promo_unique: body.promo_unique,
        like_count: 0,
        used_count: 0,
        comment_count: 0,
        active: true,
    }
    .into_db()
    .insert(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Created().json(PostPromoResponse { id: promo.id }))
}

#[derive(Serialize, Debug)]
struct PostPromoResponse {
    id: Uuid,
}
