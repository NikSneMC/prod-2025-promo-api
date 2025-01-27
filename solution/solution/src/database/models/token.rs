use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::redis::RedisConnection,
    models::{Token, TokenType},
};

use super::DatabaseError;

const TOKENS_NAMESPACE: &str = "tokens";
const TOKEN_EXPIRY: i64 = 1000 * 60 * 60 * 24;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DBToken {
    pub id: Uuid,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,

    pub issued_at: DateTime<Utc>,
}

impl DBToken {
    pub async fn insert(self, cache: &mut RedisConnection) -> Result<Self, DatabaseError> {
        let entity = self.get_entity()?;

        cache
            .set_serialized_to_json(TOKENS_NAMESPACE, entity, self.clone(), Some(TOKEN_EXPIRY))
            .await?;
        Ok(self)
    }

    pub async fn get(entity: Uuid, cache: &mut RedisConnection) -> Option<Self> {
        cache
            .get_deserialized_from_json(TOKENS_NAMESPACE, &entity.to_string())
            .await
            .unwrap_or(None)
    }

    pub async fn validate(self, cache: &mut RedisConnection) -> Result<Self, ()> {
        let entity = if let Ok(entity) = self.get_entity() {
            entity
        } else {
            return Err(());
        };

        if let Some(token) = Self::get(entity, cache).await {
            if token == self {
                return Ok(token);
            }
        }
        Err(())
    }

    pub fn get_entity(&self) -> Result<Uuid, DatabaseError> {
        Ok(*match self {
            Self {
                company_id: Some(id),
                ..
            } => id,
            Self {
                user_id: Some(id), ..
            } => id,
            _ => {
                return Err(DatabaseError::CustomCacheError(
                    "failed to cache token".to_string(),
                ))
            }
        })
    }

    pub fn into_model(self) -> Token {
        Token::from(self)
    }
}

impl From<Token> for DBToken {
    fn from(token: Token) -> Self {
        match token.token_type {
            TokenType::Company => Self {
                id: token.id,
                company_id: Some(token.entity),
                user_id: None,
                issued_at: token.issued_at,
            },
            TokenType::User => Self {
                id: token.id,
                company_id: None,
                user_id: Some(token.entity),
                issued_at: token.issued_at,
            },
        }
    }
}
