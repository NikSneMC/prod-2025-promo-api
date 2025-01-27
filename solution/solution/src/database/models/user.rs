use sqlx::{
    prelude::{FromRow, Type},
    query_file_as, Executor, Postgres, Transaction,
};
use uuid::Uuid;

use crate::models::{User, UserTargetSettings};

use super::DatabaseError;

#[derive(FromRow, Type, Debug)]
#[sqlx(type_name = "user_target_settings")]
pub struct DBUserTargetSettings {
    pub age: i32,
    pub country: String,
}

impl DBUserTargetSettings {
    pub fn into_model(self) -> UserTargetSettings {
        UserTargetSettings::from(self)
    }
}

impl From<UserTargetSettings> for DBUserTargetSettings {
    fn from(user_target_settings_model: UserTargetSettings) -> Self {
        Self {
            age: user_target_settings_model.age,
            country: user_target_settings_model.country,
        }
    }
}

#[derive(Debug)]
pub struct DBUser {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub other: DBUserTargetSettings,
    pub password_hash: String,
}

impl DBUser {
    pub async fn insert(
        self,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(
            Self,
            "sql/user/insert.sql",
            self.id,
            self.name,
            self.surname,
            self.email,
            self.avatar_url,
            self.other as DBUserTargetSettings,
            self.password_hash,
        )
        .fetch_one(&mut **transaction)
        .await?)
    }

    pub async fn get_by_id<'a, E>(id: Uuid, executor: E) -> Result<Option<Self>, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(query_file_as!(Self, "sql/user/get_by_id.sql", id)
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
        Ok(query_file_as!(Self, "sql/user/get_by_email.sql", email)
            .fetch_optional(executor)
            .await?)
    }

    pub async fn patch(
        id: Uuid,
        name: Option<&str>,
        surname: Option<&str>,
        avatar_url: Option<&str>,
        password_hash: Option<&str>,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Self, DatabaseError> {
        Ok(query_file_as!(
            Self,
            "sql/user/patch.sql",
            id,
            name,
            surname,
            avatar_url,
            password_hash
        )
        .fetch_one(&mut **transaction)
        .await?)
    }

    pub fn into_model(self) -> User {
        User::from(self)
    }
}

impl From<User> for DBUser {
    fn from(user_model: User) -> Self {
        Self {
            id: user_model.id,
            name: user_model.name,
            surname: user_model.surname,
            email: user_model.email,
            avatar_url: user_model.avatar_url,
            other: user_model.other.into_db(),
            password_hash: user_model.password_hash,
        }
    }
}
