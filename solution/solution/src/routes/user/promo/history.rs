use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

use crate::{models::Token, routes::ApiError, util::validate::validation_errors_to_string};

#[derive(Deserialize, Validate)]
struct PromosHistoryQuery {
    #[validate(range(min = 0))]
    limit: Option<u32>,

    #[validate(range(min = 0))]
    offset: Option<u32>,
}

#[get("/history")]
pub async fn get_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    query: Query<PromosHistoryQuery>,
) -> Result<HttpResponse, ApiError> {
    query
        .validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    let user = token.get_user(&**pool).await?;

    let (promos, count) = user
        .get_activation_history(
            query.limit,
            query.offset,
            &**pool,
        )
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("X-Total-Count", count))
        .json(promos))
}
