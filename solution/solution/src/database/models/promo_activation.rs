use chrono::{DateTime, Utc};
use sqlx::{query_file, query_file_as, Executor, Postgres, Transaction};
use uuid::Uuid;

use crate::models::PromoStatsCountry;

use super::{DBPromo, DBPromoMode, DBTarget, DatabaseError};

#[derive(Debug)]
pub struct DBPromoActivation {
    pub user_id: Uuid,
    pub promo_id: Uuid,
    pub promo: String,
    pub date: DateTime<Utc>,
}

impl DBPromoActivation {
    pub async fn activate_common(
        user_id: Uuid,
        promo_id: Uuid,
        date: DateTime<Utc>,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(
            Self,
            "sql/promo_activation/common.sql",
            user_id,
            promo_id,
            date
        )
        .fetch_one(&mut **transaction)
        .await?)
    }

    pub async fn activate_unique(
        user_id: Uuid,
        promo_id: Uuid,
        date: DateTime<Utc>,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(
            Self,
            "sql/promo_activation/unique.sql",
            user_id,
            promo_id,
            date
        )
        .fetch_one(&mut **transaction)
        .await?)
    }

    pub async fn get_by_ids<'a, E>(
        user_id: Uuid,
        promo_id: Uuid,
        executor: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(query_file_as!(
            Self,
            "sql/promo_activation/get_by_ids.sql",
            user_id,
            promo_id
        )
        .fetch_optional(executor)
        .await?)
    }

    pub async fn get_history<'a, E>(
        user_id: Uuid,
        limit: i64,
        offset: i64,
        executor: E,
    ) -> Result<(Vec<DBPromo>, i64), DatabaseError>
    where
        E: Executor<'a, Database = Postgres> + Copy,
    {
        Ok((
            query_file_as!(
                DBPromo,
                "sql/promo_activation/history_pageable.sql",
                user_id,
                limit,
                offset
            )
            .fetch_all(executor)
            .await?,
            query_file!("sql/promo_activation/history_count.sql", user_id)
                .fetch_one(executor)
                .await?
                .count
                .unwrap(),
        ))
    }
}

#[derive(Debug)]
pub struct DBCountryStats {
    pub country: Option<String>,
    pub activations_count: Option<i64>,
}

impl DBCountryStats {
    pub async fn get_all<'a, E>(promo_id: Uuid, executor: E) -> Result<Vec<Self>, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(
            query_file_as!(Self, "sql/promo_activation/stats.sql", promo_id,)
                .fetch_all(executor)
                .await?,
        )
    }

    pub fn into_model(self) -> PromoStatsCountry {
        PromoStatsCountry::from(self)
    }
}
