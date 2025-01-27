use actix_web::{
    delete, get, put,
    web::{Data, Json, Path, ReqData},
};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    database::models::DBComment,
    models::{Comment, CommentPath, EmptyResponse, Token},
    routes::ApiError,
    util::validate::validation_errors_to_string,
};

#[get("/{comment_id}")]
pub async fn get_handler(
    pool: Data<PgPool>,
    path: Path<CommentPath>,
) -> Result<Json<Comment>, ApiError> {
    let comment = if let Some(comment) =
        DBComment::get_by_id(path.promo_id, path.comment_id, &**pool).await?
    {
        comment
    } else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(comment.into_model()))
}

#[derive(Deserialize, Validate, Debug)]
struct UpdateCommentRequest {
    #[validate(length(min = 10, max = 1000))]
    text: String,
}

#[put("/{comment_id}")]
pub async fn put_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<CommentPath>,
    body: Json<UpdateCommentRequest>,
) -> Result<Json<Comment>, ApiError> {
    body.validate()
        .map_err(|err| ApiError::InvalidInput(validation_errors_to_string(err, None)))?;

    let comment = if let Some(comment) =
        DBComment::get_by_id(path.promo_id, path.comment_id, &**pool).await?
    {
        comment
    } else {
        return Err(ApiError::NotFound);
    };

    let user = token.get_user(&**pool).await?;

    if comment.author_id != user.id {
        return Err(ApiError::NotOwner);
    }

    let mut transaction = pool.begin().await?;

    let comment = comment
        .patch(body.text.clone(), &mut transaction)
        .await?
        .into_model();

    transaction.commit().await?;

    Ok(Json(comment))
}

#[delete("/{comment_id}")]
pub async fn delete_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
    path: Path<CommentPath>,
) -> Result<EmptyResponse, ApiError> {
    let comment = if let Some(comment) =
        DBComment::get_by_id(path.promo_id, path.comment_id, &**pool).await?
    {
        comment
    } else {
        return Err(ApiError::NotFound);
    };

    let user = token.get_user(&**pool).await?;

    if comment.author_id != user.id {
        return Err(ApiError::NotOwner);
    }

    let mut transaction = pool.begin().await?;

    comment.delete(&mut transaction).await?;

    transaction.commit().await?;

    Ok(EmptyResponse::default())
}
