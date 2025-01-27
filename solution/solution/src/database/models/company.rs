use sqlx::{query_file_as, Executor, Postgres, Transaction};
use uuid::Uuid;

use crate::models::Company;

use super::DatabaseError;

#[derive(Debug)]
pub struct DBCompany {
    pub id: Uuid,

    pub name: String,

    pub email: String,

    pub password_hash: String,
}

impl DBCompany {
    pub async fn insert(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(
            Self,
            "sql/company/insert.sql",
            self.id,
            self.name,
            self.email,
            self.password_hash,
        )
        .fetch_one(&mut **transaction)
        .await?)
    }

    pub async fn get_by_id<'a, E>(id: Uuid, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(query_file_as!(Self, "sql/company/get_by_id.sql", id)
            .fetch_optional(executor)
            .await?)
    }

    pub async fn get_by_email<'a, E>(
        email: &str,
        executor: E,
    ) -> Result<Option<Self>, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(query_file_as!(Self, "sql/company/get_by_email.sql", email)
            .fetch_optional(executor)
            .await?)
    }

    pub fn into_model(self) -> Company {
        Company::from(self)
    }
}

impl From<Company> for DBCompany {
    fn from(c: Company) -> Self {
        Self {
            id: c.id,
            name: c.name,
            email: c.email,
            password_hash: c.password_hash,
        }
    }
}
