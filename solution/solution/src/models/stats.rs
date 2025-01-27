use serde::Serialize;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::database::models::{DBCountryStats, DatabaseError};

#[derive(Serialize, Clone, Debug)]
pub struct PromoStatsCountry {
    country: String,
    activations_count: i64,
}

impl From<DBCountryStats> for PromoStatsCountry {
    fn from(db_stats: DBCountryStats) -> Self {
        Self {
            country: db_stats.country.unwrap(),
            activations_count: db_stats.activations_count.unwrap(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct PromoStats {
    pub activations_count: i64,
    pub countries: Vec<PromoStatsCountry>,
}

impl PromoStats {
    pub async fn get<'a, E>(promo_id: Uuid, executor: E) -> Result<Self, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let countries: Vec<PromoStatsCountry> = DBCountryStats::get_all(promo_id, executor)
            .await?
            .into_iter()
            .map(DBCountryStats::into_model)
            .collect();

        let activations_count: i64 = countries
            .clone()
            .into_iter()
            .map(|c| c.activations_count)
            .sum();

        Ok(Self {
            activations_count,
            countries,
        })
    }
}
