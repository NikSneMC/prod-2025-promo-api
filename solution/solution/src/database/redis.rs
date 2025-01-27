use crate::{REDIS_HOST, REDIS_PORT};

use super::models::DatabaseError;
use deadpool_redis::{Config, Connection, Pool, Runtime};
use redis::{cmd, Cmd};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

const DEFAULT_EXPIRY: i64 = 60 * 60 * 12;

#[derive(Clone)]
pub struct RedisPool {
    pub pool: Pool,
    meta_namespace: String,
}

pub struct RedisConnection {
    pub connection: Connection,
    meta_namespace: String,
}

impl RedisPool {
    pub fn new(meta_namespace: Option<String>) -> Self {
        let redis_pool = Config::from_url(format!("redis://{}:{}/0", REDIS_HOST(), REDIS_PORT()))
            .builder()
            .expect("Error building Redis pool")
            .max_size(10000)
            .runtime(Runtime::Tokio1)
            .build()
            .expect("Redis connection failed");

        RedisPool {
            pool: redis_pool,
            meta_namespace: meta_namespace.unwrap_or("".to_string()),
        }
    }

    pub async fn connect(&self) -> Result<RedisConnection, DatabaseError> {
        Ok(RedisConnection {
            connection: self.pool.get().await?,
            meta_namespace: self.meta_namespace.clone(),
        })
    }
}

impl RedisConnection {
    pub async fn set(
        &mut self,
        namespace: &str,
        id: &str,
        data: &str,
        expiry: Option<i64>,
    ) -> Result<(), DatabaseError> {
        let mut cmd = cmd("SET");
        redis_args(
            &mut cmd,
            vec![
                format!("{}_{}:{}", self.meta_namespace, namespace, id),
                data.to_string(),
                "EX".to_string(),
                expiry.unwrap_or(DEFAULT_EXPIRY).to_string(),
            ]
            .as_slice(),
        );
        redis_execute::<()>(&mut cmd, &mut self.connection).await?;
        Ok(())
    }

    pub async fn set_serialized_to_json<Id, D>(
        &mut self,
        namespace: &str,
        id: Id,
        data: D,
        expiry: Option<i64>,
    ) -> Result<(), DatabaseError>
    where
        Id: Display,
        D: serde::Serialize,
    {
        self.set(
            namespace,
            &id.to_string(),
            &serde_json::to_string(&data)?,
            expiry,
        )
        .await
    }

    pub async fn get(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<Option<String>, DatabaseError> {
        let mut cmd = cmd("GET");
        redis_args(
            &mut cmd,
            vec![format!("{}_{}:{}", self.meta_namespace, namespace, id)].as_slice(),
        );
        let res = redis_execute(&mut cmd, &mut self.connection).await?;
        Ok(res)
    }

    pub async fn get_many(
        &mut self,
        namespace: &str,
        ids: &[String],
    ) -> Result<Vec<Option<String>>, DatabaseError> {
        let mut cmd = cmd("MGET");
        redis_args(
            &mut cmd,
            ids.iter()
                .map(|x| format!("{}_{}:{}", self.meta_namespace, namespace, x))
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let res = redis_execute(&mut cmd, &mut self.connection).await?;
        Ok(res)
    }

    pub async fn get_deserialized_from_json<R>(
        &mut self,
        namespace: &str,
        id: &str,
    ) -> Result<Option<R>, DatabaseError>
    where
        R: for<'a> serde::Deserialize<'a>,
    {
        Ok(self
            .get(namespace, id)
            .await?
            .and_then(|x| serde_json::from_str(&x).ok()))
    }

    pub async fn get_many_deserialized_from_json<R>(
        &mut self,
        namespace: &str,
        ids: &[String],
    ) -> Result<Vec<Option<R>>, DatabaseError>
    where
        R: for<'a> serde::Deserialize<'a>,
    {
        Ok(self
            .get_many(namespace, ids)
            .await?
            .into_iter()
            .map(|x| x.and_then(|val| serde_json::from_str::<R>(&val).ok()))
            .collect::<Vec<_>>())
    }

    pub async fn delete<T1>(&mut self, namespace: &str, id: T1) -> Result<(), DatabaseError>
    where
        T1: Display,
    {
        let mut cmd = cmd("DEL");
        redis_args(
            &mut cmd,
            vec![format!("{}_{}:{}", self.meta_namespace, namespace, id)].as_slice(),
        );
        redis_execute::<()>(&mut cmd, &mut self.connection).await?;
        Ok(())
    }

    pub async fn delete_many(
        &mut self,
        iter: impl IntoIterator<Item = (&str, Option<String>)>,
    ) -> Result<(), DatabaseError> {
        let mut cmd = cmd("DEL");
        let mut any = false;
        for (namespace, id) in iter {
            if let Some(id) = id {
                redis_args(
                    &mut cmd,
                    [format!("{}_{}:{}", self.meta_namespace, namespace, id)].as_slice(),
                );
                any = true;
            }
        }

        if any {
            redis_execute::<()>(&mut cmd, &mut self.connection).await?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct RedisValue<T, K, S> {
    key: K,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<S>,
    iat: i64,
    val: T,
}

pub fn redis_args(cmd: &mut Cmd, args: &[String]) {
    for arg in args {
        cmd.arg(arg);
    }
}

pub async fn redis_execute<T>(
    cmd: &mut Cmd,
    redis: &mut deadpool_redis::Connection,
) -> Result<T, deadpool_redis::PoolError>
where
    T: redis::FromRedisValue,
{
    let res = cmd.query_async::<T>(redis).await?;
    Ok(res)
}
