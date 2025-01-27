use actix_web::{
    delete, post,
    web::{Data, Path, ReqData},
};
use sqlx::PgPool;

use crate::{
    database::models::DBPromo,
    models::{EmptyResponse, PromoPath, Token},
    routes::ApiError,
};

#[post("like")]
pub async fn post_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<PromoPath>,
) -> Result<EmptyResponse, ApiError> {
    let promo = if let Some(promo) = DBPromo::get_by_id(path.promo_id, &**pool).await? {
        promo
    } else {
        return Err(ApiError::NotFound);
    }
    .into_model()
    .into_user(token.entity, &**pool)
    .await?;

    promo.like(&**pool).await?;

    Ok(EmptyResponse::default())
}

#[delete("like")]
pub async fn delete_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<PromoPath>,
) -> Result<EmptyResponse, ApiError> {
    let promo = if let Some(promo) = DBPromo::get_by_id(path.promo_id, &**pool).await? {
        promo
    } else {
        return Err(ApiError::NotFound);
    }
    .into_model()
    .into_user(token.entity, &**pool)
    .await?;

    promo.unlike(&**pool).await?;

    Ok(EmptyResponse::default())
}
