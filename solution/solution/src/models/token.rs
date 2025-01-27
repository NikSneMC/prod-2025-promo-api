use crate::{
    auth::AuthenticationError,
    database::{
        models::{DBCompany, DBToken, DBUser, DatabaseError},
        redis::RedisConnection,
    },
    util::convertions::{decode_string, decode_uuid},
};
use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, GeneralPurpose},
    Engine,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

use super::{Company, User};

#[derive(
    Deserialize, Serialize, Clone, Display, PartialEq, Eq, EnumString, IntoStaticStr, Debug,
)]
pub enum TokenType {
    #[strum(serialize = "cmp")]
    Company,
    #[strum(serialize = "usr")]
    User,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Token {
    pub id: Uuid,
    pub token_type: TokenType,
    pub entity: Uuid,
    pub issued_at: DateTime<Utc>,
}

impl Token {
    const ENGINE: GeneralPurpose =
        GeneralPurpose::new(&URL_SAFE, NO_PAD.with_decode_allow_trailing_bits(true));

    pub fn new(token_type: TokenType, entity: Uuid) -> Self {
        Self {
            id: Uuid::now_v7(),
            token_type,
            entity,
            issued_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() - self.issued_at >= chrono::Duration::days(1)
    }

    pub async fn validate(self, cache: &mut RedisConnection) -> Result<Self, AuthenticationError> {
        Ok(self
            .into_db()
            .validate(cache)
            .await
            .map_err(|_| AuthenticationError::InvalidCredentials)?
            .into_model())
    }

    pub async fn get_company<'a, E>(&self, executor: E) -> Result<Company, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(DBCompany::get_by_id(self.entity, executor)
            .await?
            .unwrap()
            .into_model())
    }

    pub async fn get_user<'a, E>(&self, executor: E) -> Result<User, DatabaseError>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(DBUser::get_by_id(self.entity, executor)
            .await?
            .unwrap()
            .into_model())
    }

    pub fn from_str(value: &str) -> Result<Self, ()> {
        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() != 4 {
            return Err(());
        }

        let id = decode_uuid(&Self::ENGINE, parts[0])?;

        let token_type = parts[1].to_owned().parse().map_err(|_| ())?;

        let entity = decode_uuid(&Self::ENGINE, parts[2])?;

        let issued_at = decode_string(&Self::ENGINE, parts[3])?
            .parse::<i64>()
            .map_err(|_| ())?;
        if let Some(issued_at) = DateTime::<Utc>::from_timestamp_millis(issued_at) {
            Ok(Self {
                id,
                token_type,
                entity,
                issued_at,
            })
        } else {
            Err(())
        }
    }

    pub fn into_db(self) -> DBToken {
        DBToken::from(self)
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            Self::ENGINE.encode(&self.id),
            self.token_type.to_string(),
            Self::ENGINE.encode(&self.entity),
            Self::ENGINE.encode(self.issued_at.timestamp_millis().to_string())
        )
    }
}

impl From<DBToken> for Token {
    fn from(db_token: DBToken) -> Self {
        match db_token {
            DBToken {
                id,
                company_id: Some(entity),
                issued_at,
                ..
            } => Self {
                id,
                token_type: TokenType::Company,
                entity,
                issued_at,
            },
            DBToken {
                id,
                user_id: Some(entity),
                issued_at,
                ..
            } => Self {
                id,
                token_type: TokenType::User,
                entity,
                issued_at,
            },
            _ => unreachable!(),
        }
    }
}
