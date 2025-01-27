use actix_web::{
    get,
    web::{Data, Json, ReqData},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    models::{Token, UserTargetSettings},
    routes::ApiError,
};

#[get("")]
pub async fn get_handler(
    pool: Data<PgPool>,
    token: ReqData<Token>,
) -> Result<Json<GetProfileResponse>, ApiError> {
    let user = token.get_user(&**pool).await?;

    Ok(Json(GetProfileResponse {
        name: user.name,
        surname: user.surname,
        email: user.email,
        avatar_url: user.avatar_url,
        other: user.other,
    }))
}

#[derive(Serialize, Debug)]
pub struct GetProfileResponse {
    name: String,
    surname: String,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    other: UserTargetSettings,
}
