use thiserror::Error;

mod comment;
mod company;
mod like;
mod promo;
mod promo_activation;
mod token;
mod user;

pub use comment::DBComment;
pub use company::DBCompany;
pub use like::DBLike;
pub use promo::{DBPromo, DBPromoMode, DBTarget};
pub use promo_activation::{DBCountryStats, DBPromoActivation};
pub use token::DBToken;
pub use user::{DBUser, DBUserTargetSettings};

pub struct Email(pub String);

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error while interacting with the database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Error while interacting with the cache: {0}")]
    CacheError(#[from] redis::RedisError),

    #[error("Error while interacting with the cache: {0}")]
    CustomCacheError(String),

    #[error("Redis Pool Error: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),

    #[error("Error while serializing with the cache: {0}")]
    SerdeCacheError(#[from] serde_json::Error),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("Timeout when waiting for cache subscriber")]
    CacheTimeout,
}
