use futures::{stream::FuturesOrdered, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use uuid::Uuid;
use validator::Validate;

use crate::{
    database::models::{DBPromoActivation, DBUser, DBUserTargetSettings},
    routes::ApiError,
    util::validate::validate_country,
};

use super::{comment::CommentAuthor, PromoTarget, UserPromo};

#[derive(Deserialize, Serialize, Validate, Clone, Debug)]
pub struct UserTargetSettings {
    #[validate(range(min = 0, max = 100))]
    pub age: i32,

    #[validate(custom(function = "validate_country"))]
    pub country: String,
}

impl UserTargetSettings {
    pub fn into_db(self) -> DBUserTargetSettings {
        DBUserTargetSettings::from(self)
    }
}

impl From<DBUserTargetSettings> for UserTargetSettings {
    fn from(db_user_target_settings: DBUserTargetSettings) -> Self {
        Self {
            age: db_user_target_settings.age,
            country: db_user_target_settings.country,
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub other: UserTargetSettings,
    pub password_hash: String,
}

impl User {
    pub fn matches_target(&self, target: PromoTarget) -> bool {
        let mut matches = true;

        matches &= target.country.is_none() || target.country.unwrap() == self.other.country;

        matches &= target.age_from.is_none() || target.age_from.unwrap() <= self.other.age;
        matches &= target.age_until.is_none() || target.age_until.unwrap() >= self.other.age;

        matches
    }

    pub async fn get_activation_history<'a, E>(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        executor: E,
    ) -> Result<(Vec<UserPromo>, i64), ApiError>
    where
        E: Executor<'a, Database = Postgres> + Copy,
    {
        let limit: i64 = match limit {
            Some(limit) if limit > 57 => 57,
            Some(limit) => limit.into(),
            None => 10,
        };

        let (db_promos, count) =
            DBPromoActivation::get_history(self.id, limit, offset.unwrap_or(0).into(), executor)
                .await?;

        let promos: Vec<UserPromo> = db_promos
            .into_iter()
            .map(|p| p.into_model().into_user(self.id, executor))
            .collect::<FuturesOrdered<_>>()
            .try_collect()
            .await?;

        Ok((promos, count))
    }

    pub fn into_db(self) -> DBUser {
        DBUser::from(self)
    }

    pub fn into_comment_author(self) -> CommentAuthor {
        CommentAuthor::from(self)
    }
}

impl From<DBUser> for User {
    fn from(db_user: DBUser) -> Self {
        Self {
            id: db_user.id,
            name: db_user.name,
            surname: db_user.surname,
            email: db_user.email.to_string(),
            avatar_url: db_user.avatar_url.map(|url| url.to_string()),
            other: db_user.other.into_model(),
            password_hash: db_user.password_hash,
        }
    }
}
