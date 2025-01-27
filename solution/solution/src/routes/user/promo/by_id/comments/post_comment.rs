use actix_web::{
    post,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    database::models::DBPromo,
    models::{Comment, PromoPath, Token},
    routes::ApiError,
    util::validate::validation_errors_to_string,
};

#[derive(Deserialize, Validate, Debug)]
struct PostCommentRequest {
    #[validate(length(min = 10, max = 1000))]
    text: String,
}

#[post("")]
pub async fn post_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<PromoPath>,
    Json(body): Json<PostCommentRequest>,
) -> Result<HttpResponse, ApiError> {
    body.validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    let promo = if let Some(promo) = DBPromo::get_by_id(path.promo_id, &**pool).await? {
        promo
    } else {
        return Err(ApiError::NotFound);
    };

    let user = token.get_user(&**pool).await?;

    let mut transaction = pool.begin().await?;

    let commment = Comment {
        id: Uuid::now_v7(),
        author_id: user.id,
        promo_id: promo.id,
        text: body.text,
        date: Utc::now(),
        author: user.into_comment_author(),
    }
    .into_db()
    .insert(&mut transaction)
    .await?
    .into_model();

    transaction.commit().await?;

    Ok(HttpResponse::Created().json(commment))
}
