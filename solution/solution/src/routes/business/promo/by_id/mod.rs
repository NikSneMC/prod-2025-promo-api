use actix_web::{
    get,
    middleware::from_fn,
    patch,
    web::{scope, Data, Json, Path, ReqData, ServiceConfig},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::auth_middleware_cmp,
    database::models::{DBPromo, DBPromoMode},
    models::{Promo, PromoPath, PromoTarget, Token},
    routes::ApiError,
    util::{
        convertions::promo_date_format, cors::default_cors, validate::validation_errors_to_string,
    },
};

mod stat;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("{promo_id}")
            .wrap(default_cors())
            .wrap(from_fn(auth_middleware_cmp))
            .service(get_handler)
            .service(patch_handler)
            .service(stat::get_handler),
    );
}

#[get("")]
pub async fn get_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<PromoPath>,
) -> Result<Json<Promo>, ApiError> {
    let promo = if let Some(promo) = DBPromo::get_by_id(path.promo_id, &**pool).await? {
        promo
    } else {
        return Err(ApiError::NotFound);
    };

    let company = token.get_company(&**pool).await?;

    if promo.company_id != company.id {
        return Err(ApiError::NotOwner);
    }

    Ok(Json(promo.into_model()))
}

#[derive(Deserialize, Validate, Debug)]
struct EditPromoRequest {
    #[validate(length(min = 10, max = 300))]
    description: Option<String>,

    #[validate(length(max = 350))]
    image_url: Option<String>,

    #[validate(nested)]
    target: Option<PromoTarget>,

    #[validate(range(min = 0, max = 100000000))]
    max_count: Option<i32>,

    #[serde(default, with = "promo_date_format")]
    active_from: Option<DateTime<Utc>>,

    #[serde(default, with = "promo_date_format")]
    active_until: Option<DateTime<Utc>>,
}

#[patch("")]
pub async fn patch_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<PromoPath>,
    body: Json<EditPromoRequest>,
) -> Result<Json<Promo>, ApiError> {
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

    let promo = if let Some(promo) = DBPromo::get_by_id(path.promo_id, &**pool).await? {
        promo
    } else {
        return Err(ApiError::NotFound);
    };

    if let Some(max_count) = body.max_count {
        if promo.mode == DBPromoMode::UNIQUE && max_count != 1 {
            return Err(ApiError::InvalidInput(
                "field `max_count` must be 1 for UNIQUE promocodes".to_string(),
            ));
        }
        if promo.used_count > max_count {
            return Err(ApiError::InvalidInput(
                "field `max_count` must be greater than current number of activations".to_string(),
            ));
        }
    }

    let company = token.get_company(&**pool).await?;

    if promo.company_id != company.id {
        return Err(ApiError::NotOwner);
    }

    let mut transaction = pool.begin().await?;

    let promo = promo
        .patch(
            body.description.clone(),
            body.image_url.clone(),
            body.target.clone().map(PromoTarget::into_db),
            body.max_count,
            body.active_from,
            body.active_until,
            &mut transaction,
        )
        .await?
        .into_model();

    transaction.commit().await?;

    Ok(Json(promo))
}
