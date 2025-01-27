use actix_web::{
    post,
    web::{Data, Json, Path, ReqData},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    database::{models::DBPromo, redis::RedisPool},
    models::{PromoPath, Token},
    routes::ApiError,
};

#[post("activate")]
pub async fn post_handler(
    pool: Data<PgPool>,
    cache: Data<RedisPool>,
    token: ReqData<Token>,
    path: Path<PromoPath>,
) -> Result<Json<ActivatePromoResponse>, ApiError> {
    let promo = if let Some(promo) = DBPromo::get_by_id(path.promo_id, &**pool).await? {
        promo
    } else {
        return Err(ApiError::NotFound);
    }
    .into_model();

    if !promo.active {
        return Err(ApiError::PromoExpired);
    }

    let user = token.get_user(&**pool).await?;

    if !user.matches_target(promo.target.clone()) {
        return Err(ApiError::NotPromoTarget);
    }

    let promo = promo.get_code(&user, &**pool, &cache).await?;

    Ok(Json(ActivatePromoResponse { promo }))
}

#[derive(Serialize, Debug)]
struct ActivatePromoResponse {
    promo: String,
}
