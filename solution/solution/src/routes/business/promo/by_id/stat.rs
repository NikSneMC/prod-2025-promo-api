use actix_web::{
    get,
    web::{Data, Json, Path, ReqData},
};
use sqlx::PgPool;

use crate::{
    database::models::DBPromo,
    models::{PromoPath, PromoStats, Token},
    routes::ApiError,
};

#[get("/stat")]
pub async fn get_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<PromoPath>,
) -> Result<Json<PromoStats>, ApiError> {
    let promo = if let Some(promo) = DBPromo::get_by_id(path.promo_id, &**pool).await? {
        promo
    } else {
        return Err(ApiError::NotFound);
    };

    let company = token.get_company(&**pool).await?;

    if promo.company_id != company.id {
        return Err(ApiError::NotOwner);
    }

    let stats = PromoStats::get(promo.id, &**pool).await?;

    Ok(Json(stats))
}
