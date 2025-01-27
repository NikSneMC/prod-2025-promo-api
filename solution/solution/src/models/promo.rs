use crate::{
    database::{
        models::{DBLike, DBPromo, DBPromoActivation, DBPromoMode, DBTarget, DatabaseError},
        redis::RedisPool,
    },
    routes::ApiError,
    util::{
        antifraud,
        convertions::serialize_opt_promo_date,
        validate::{validate_country, validate_target},
        values::{MAX_DATETIME, MIN_DATETIME},
    },
};
use chrono::{DateTime, Utc};
use futures::{stream::FuturesOrdered, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;
use validator::Validate;

use super::User;

#[derive(Deserialize, Validate, Debug)]
pub struct PromoPath {
    pub promo_id: Uuid,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SortPromosBy {
    ActiveFrom,
    ActiveUntil,
}

#[derive(Deserialize, Serialize, Validate, Clone, Debug)]
#[validate(schema(function = "validate_target"))]
pub struct PromoTarget {
    #[validate(range(min = 0, max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_from: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 0, max = 100))]
    pub age_until: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "validate_country"))]
    pub country: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 20))]
    pub categories: Option<Vec<String>>,
}

impl PromoTarget {
    pub fn into_db(self) -> DBTarget {
        DBTarget::from(self)
    }
}

impl From<DBTarget> for PromoTarget {
    fn from(value: DBTarget) -> Self {
        Self {
            age_from: value.age_from,
            age_until: value.age_until,
            country: value.country,
            categories: value.categories,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Promo {
    #[serde(rename = "promo_id")]
    pub id: Uuid,

    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,

    pub target: PromoTarget,

    pub max_count: i32,

    #[serde(
        serialize_with = "serialize_opt_promo_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub active_from: Option<DateTime<Utc>>,

    #[serde(
        serialize_with = "serialize_opt_promo_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub active_until: Option<DateTime<Utc>>,

    pub mode: DBPromoMode,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub promo_common: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub promo_unique: Option<Vec<String>>,

    pub company_id: Uuid,

    pub company_name: String,

    pub like_count: i32,

    pub used_count: i32,

    pub comment_count: i32,

    pub active: bool,
}

impl Promo {
    pub async fn get_pageable<'a, E>(
        company_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
        sort_by: &Option<SortPromosBy>,
        countries: &Option<Vec<String>>,
        executor: E,
    ) -> Result<(Vec<Self>, i64), DatabaseError>
    where
        E: Executor<'a, Database = Postgres> + Copy,
    {
        let limit: i64 = match limit {
            Some(limit) if limit > 57 => 57,
            Some(limit) => limit.into(),
            None => 10,
        };

        DBPromo::get_pageable(
            company_id,
            limit,
            offset.unwrap_or(0).into(),
            &sort_by
                .map(|v| serde_json::to_string(&v).unwrap())
                .unwrap_or("id".to_string()),
            &countries
                .clone()
                .map(|v| v.into_iter().map(|s| s.to_lowercase()).collect()),
            executor,
        )
        .await
        .map(|v| (v.0.into_iter().map(DBPromo::into_model).collect(), v.1))
    }

    pub async fn get_code(
        self,
        user: &User,
        pool: &PgPool,
        cache: &RedisPool,
    ) -> Result<String, ApiError> {
        if !antifraud::ask(user.email.clone(), self.id, cache).await? {
            return Err(ApiError::FraudDetected);
        }

        let mut transaction = pool.begin().await?;

        let date = Utc::now();

        let promo = match self.mode {
            DBPromoMode::COMMON => {
                DBPromoActivation::activate_common(user.id, self.id, date, &mut transaction).await
            }
            DBPromoMode::UNIQUE => {
                DBPromoActivation::activate_unique(user.id, self.id, date, &mut transaction).await
            }
        }?
        .promo;

        transaction.commit().await?;

        Ok(promo)
    }

    pub fn into_db(self) -> DBPromo {
        DBPromo::from(self)
    }
}

impl From<DBPromo> for Promo {
    fn from(db_promo: DBPromo) -> Self {
        let active_from = if db_promo.active_from <= MIN_DATETIME {
            None
        } else {
            Some(db_promo.active_from)
        };

        let active_until = if db_promo.active_until >= MAX_DATETIME {
            None
        } else {
            Some(db_promo.active_until)
        };

        let promo_unique = if db_promo.promo_unique.is_empty() {
            None
        } else {
            Some(db_promo.promo_unique)
        };

        Self {
            id: db_promo.id,
            description: db_promo.description,
            image_url: db_promo.image_url,
            target: db_promo.target.into(),
            max_count: db_promo.max_count,
            active_from,
            active_until,
            mode: db_promo.mode,
            promo_common: db_promo.promo_common,
            promo_unique,
            company_id: db_promo.company_id,
            company_name: db_promo.company_name.unwrap_or_default(),
            like_count: db_promo.like_count,
            used_count: db_promo.used_count,
            comment_count: db_promo.comment_count,
            active: db_promo.active,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct UserPromo {
    #[serde(skip)]
    user_id: Uuid,

    promo_id: Uuid,

    company_id: Uuid,

    company_name: String,

    description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,

    active: bool,

    is_activated_by_user: bool,

    like_count: i32,

    is_liked_by_user: bool,

    comment_count: i32,
}

impl UserPromo {
    pub async fn get_pageable<'a, E>(
        user: &User,
        limit: Option<u32>,
        offset: Option<u32>,
        category: Option<&str>,
        active: Option<bool>,
        executor: E,
    ) -> Result<(Vec<Self>, i64), ApiError>
    where
        E: Executor<'a, Database = Postgres> + Copy,
    {
        let limit: i64 = match limit {
            Some(limit) if limit > 57 => 57,
            Some(limit) => limit.into(),
            None => 10,
        };

        let (db_promos, count) = DBPromo::get_pageable_user(
            limit,
            offset.unwrap_or(0).into(),
            &user.other.country,
            category,
            active,
            executor,
        )
        .await?;

        let promos: Vec<Self> = db_promos
            .into_iter()
            .map(|db_promo| {
                let promo = db_promo.into_model();
                async move { promo.into_user(user.id, executor).await }
            })
            .collect::<FuturesOrdered<_>>()
            .try_collect()
            .await?;

        Ok((promos, count))
    }

    pub async fn like(&self, pool: &PgPool) -> Result<(), ApiError> {
        if DBLike::get(self.user_id, self.promo_id, &*pool)
            .await?
            .is_some()
        {
            return Ok(());
        }

        let mut transaction = pool.begin().await?;

        DBLike {
            user_id: self.user_id,
            promo_id: self.promo_id,
        }
        .insert(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn unlike(&self, pool: &PgPool) -> Result<(), ApiError> {
        let like = if let Some(like) = DBLike::get(self.user_id, self.promo_id, &*pool).await? {
            like
        } else {
            return Ok(());
        };

        let mut transaction = pool.begin().await?;

        like.delete(&mut transaction).await?;

        transaction.commit().await?;

        Ok(())
    }
}

impl Promo {
    pub async fn into_user<'a, E>(self, user_id: Uuid, executor: E) -> Result<UserPromo, ApiError>
    where
        E: Executor<'a, Database = Postgres> + Copy,
    {
        let is_activated_by_user = DBPromoActivation::get_by_ids(user_id, self.id, executor)
            .await?
            .is_some();

        let is_liked_by_user = DBLike::get(user_id, self.id, executor).await?.is_some();

        Ok(UserPromo {
            user_id,
            promo_id: self.id,
            company_id: self.company_id,
            company_name: self.company_name,
            description: self.description,
            image_url: self.image_url,
            active: self.active,
            is_activated_by_user,
            like_count: self.like_count,
            is_liked_by_user,
            comment_count: self.comment_count,
        })
    }
}
