use actix_web::{
    get,
    web::{Data, Json, Path, ReqData},
};
use sqlx::PgPool;

use crate::{
    database::models::DBPromo,
    models::{PromoPath, Token, UserPromo},
    routes::ApiError,
};

#[get("")]
pub async fn get_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<PromoPath>,
) -> Result<Json<UserPromo>, ApiError> {
    let promo = if let Some(promo) = DBPromo::get_by_id(path.promo_id, &**pool).await? {
        promo
    } else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(
        promo.into_model().into_user(token.entity, &**pool).await?,
    ))
}
