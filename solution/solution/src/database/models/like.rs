use sqlx::{query_file_as, Executor, Postgres, Transaction};
use uuid::Uuid;

use super::DatabaseError;

#[derive(Debug)]
pub struct DBLike {
    pub user_id: Uuid,
    pub promo_id: Uuid,
}

impl DBLike {
    pub async fn get<'a, E>(
        user_id: Uuid,
        promo_id: Uuid,
        executor: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(query_file_as!(Self, "sql/like/get.sql", user_id, promo_id)
            .fetch_optional(executor)
            .await?)
    }

    pub async fn insert(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(
            query_file_as!(Self, "sql/like/insert.sql", self.user_id, self.promo_id)
                .fetch_one(&mut **transaction)
                .await?,
        )
    }

    pub async fn delete(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), DatabaseError> {
        query_file_as!(Self, "sql/like/delete.sql", self.user_id, self.promo_id)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }
}
