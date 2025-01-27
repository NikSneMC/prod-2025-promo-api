use std::i64;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    query_file, query_file_as, Executor, Postgres, Transaction,
};
use uuid::Uuid;

use crate::{
    models::{Promo, PromoTarget},
    util::values::{MAX_DATETIME, MIN_DATETIME},
};

use super::DatabaseError;

#[derive(FromRow, Type, Debug)]
#[sqlx(type_name = "target")]
pub struct DBTarget {
    pub age_from: Option<i32>,
    pub age_until: Option<i32>,
    pub country: Option<String>,
    pub categories: Option<Vec<String>>,
}

impl From<PromoTarget> for DBTarget {
    fn from(promo: PromoTarget) -> Self {
        Self {
            age_from: promo.age_from,
            age_until: promo.age_until,
            country: promo.country,
            categories: promo.categories,
        }
    }
}

#[derive(Type, Deserialize, Serialize, PartialEq, Eq, Debug)]
#[sqlx(type_name = "promo_mode")]
pub enum DBPromoMode {
    COMMON,
    UNIQUE,
}

#[derive(Debug)]
pub struct DBPromo {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: Option<String>,
    pub description: String,
    pub image_url: Option<String>,
    pub target: DBTarget,
    pub max_count: i32,
    pub active_from: DateTime<Utc>,
    pub active_until: DateTime<Utc>,
    pub mode: DBPromoMode,
    pub promo_common: Option<String>,
    pub promo_unique: Vec<String>,
    pub like_count: i32,
    pub used_count: i32,
    pub comment_count: i32,
    pub active: bool,
}

impl DBPromo {
    pub async fn insert(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(
            Self,
            "sql/promo/insert.sql",
            self.id,
            self.company_id,
            self.description,
            self.image_url,
            self.target as DBTarget,
            self.max_count,
            self.active_from,
            self.active_until,
            self.mode as DBPromoMode,
            self.promo_common,
            &self.promo_unique,
            self.like_count,
            self.used_count,
            self.comment_count,
            self.active
        )
        .fetch_one(&mut **transaction)
        .await?)
    }

    pub async fn get_pageable<'a, E>(
        company_id: Uuid,
        limit: i64,
        offset: i64,
        sort_by: &str,
        countries: &Option<Vec<String>>,
        executor: E,
    ) -> Result<(Vec<DBPromo>, i64), DatabaseError>
    where
        E: Executor<'a, Database = Postgres> + Copy,
    {
        let promos = match (countries, sort_by) {
            (Some(countries), "id") => {
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_id_by_countries.sql",
                    company_id,
                    countries,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await
            }
            (Some(countries), r#""active_from""#) => {
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_active_from_by_countries.sql",
                    company_id,
                    countries,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await
            }
            (Some(countries), r#""active_until""#) => {
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_active_until_by_countries.sql",
                    company_id,
                    countries,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await
            }
            (None, "id") => {
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_id.sql",
                    company_id,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await
            }
            (None, r#""active_from""#) => {
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_active_from.sql",
                    company_id,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await
            }
            (None, r#""active_until""#) => {
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_active_until.sql",
                    company_id,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await
            }
            _ => unreachable!(),
        }?;
        let count = if let Some(countries) = countries {
            query_file!("sql/promo/count_by_countries.sql", company_id, countries)
                .fetch_one(executor)
                .await?
                .count
        } else {
            query_file!("sql/promo/count.sql", company_id)
                .fetch_one(executor)
                .await?
                .count
        }
        .unwrap();

        return Ok((promos, count));
    }

    pub async fn get_pageable_user<'a, E>(
        limit: i64,
        offset: i64,
        country: &str,
        category: Option<&str>,
        active: Option<bool>,
        executor: E,
    ) -> Result<(Vec<Self>, i64), DatabaseError>
    where
        E: Executor<'a, Database = Postgres> + Copy,
    {
        let (promos, count) = match (category, active) {
            (Some(category), Some(active)) => (
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_user_by_category_active.sql",
                    country,
                    category,
                    active,
                    limit,
                    offset,
                )
                .fetch_all(executor)
                .await?,
                query_file!(
                    "sql/promo/count_user_by_category_active.sql",
                    country,
                    category,
                    active
                )
                .fetch_one(executor)
                .await?
                .count,
            ),
            (Some(category), None) => (
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_user_by_category.sql",
                    country,
                    category,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await?,
                query_file!("sql/promo/count_user_by_category.sql", country, category)
                    .fetch_one(executor)
                    .await?
                    .count,
            ),
            (None, Some(active)) => (
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_user_by_active.sql",
                    country,
                    active,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await?,
                query_file!("sql/promo/count_user_by_active.sql", country, active)
                    .fetch_one(executor)
                    .await?
                    .count,
            ),
            (None, None) => (
                query_file_as!(
                    Self,
                    "sql/promo/get_pageable_user.sql",
                    country,
                    limit,
                    offset
                )
                .fetch_all(executor)
                .await?,
                query_file!("sql/promo/count_user.sql", country)
                    .fetch_one(executor)
                    .await?
                    .count,
            ),
        };
        Ok((promos, count.unwrap()))
    }

    pub async fn get_by_id<'a, E>(id: Uuid, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(query_file_as!(Self, "sql/promo/get_by_id.sql", id)
            .fetch_optional(executor)
            .await?)
    }

    pub async fn patch(
        self,
        description: Option<String>,
        image_url: Option<String>,
        target: Option<DBTarget>,
        max_count: Option<i32>,
        active_from: Option<DateTime<Utc>>,
        active_until: Option<DateTime<Utc>>,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(
            Self,
            "sql/promo/patch.sql",
            self.id,
            description,
            image_url,
            target as Option<DBTarget>,
            max_count,
            active_from,
            active_until
        )
        .fetch_one(&mut **transaction)
        .await?)
    }

    pub fn into_model(self) -> Promo {
        Promo::from(self)
    }
}

impl From<Promo> for DBPromo {
    fn from(promo: Promo) -> Self {
        Self {
            id: promo.id,
            company_id: promo.company_id,
            company_name: Some(promo.company_name),
            description: promo.description,
            image_url: promo.image_url,
            target: promo.target.into(),
            max_count: promo.max_count,
            active_from: promo.active_from.unwrap_or(MIN_DATETIME),
            active_until: promo.active_until.unwrap_or(MAX_DATETIME),
            mode: promo.mode,
            promo_common: promo.promo_common,
            promo_unique: promo.promo_unique.unwrap_or(vec![]),
            like_count: promo.like_count,
            used_count: promo.used_count,
            comment_count: promo.comment_count,
            active: promo.active,
        }
    }
}
