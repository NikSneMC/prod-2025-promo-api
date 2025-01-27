use actix_web::web::{get, Data, JsonConfig, PathConfig, ServiceConfig};
use database::redis::RedisPool;
use log::{info, warn};
use scheduler::{update_promos_active, Scheduler};
use sqlx::{Pool, Postgres};
use std::sync::OnceLock;
use std::time::Duration;
use std::{env, sync::Arc};

use crate::routes::{not_found, ApiError};

pub mod auth;
pub mod database;
pub mod models;
pub mod routes;
pub mod scheduler;
pub mod util;

environment_variables! {
    SERVER_ADDRESS: "0.0.0.0:8080",
    POSTGRES_CONN: "postgres://postgres:postgres@localhost:5432/promo-code-backend-prod",
    REDIS_HOST: "localhost",
    REDIS_PORT: "6379",
    ANTIFRAUD_ADDRESS: "localhost:9090",
}

#[derive(Clone)]
pub struct SolutionConfig {
    pub postgres_pool: Pool<Postgres>,
    pub redis_pool: RedisPool,
    pub scheduler: Arc<Scheduler>,
}

pub fn app_setup(pool: Pool<Postgres>, redis_pool: RedisPool) -> SolutionConfig {
    info!("Starting Solution on {}", SERVER_ADDRESS());

    let mut scheduler = Scheduler::new();

    let pool_ref = pool.clone();
    scheduler.run(Duration::from_secs(60 * 60 * 24), move || {
        let pool_ref = pool_ref.clone();
        async move {
            info!("Updating `active` field on promos");
            let result = update_promos_active(&pool_ref).await;
            if let Err(e) = result {
                warn!("Updating `active` field failed: {:?}", e);
            }
            info!("Done updating `active` field on promos");
        }
    });

    SolutionConfig {
        postgres_pool: pool,
        redis_pool,
        scheduler: Arc::new(scheduler),
    }
}

pub fn solution_config(solution_config: SolutionConfig) -> impl FnOnce(&mut ServiceConfig) {
    move |cfg: &mut ServiceConfig| {
        cfg.app_data(
            PathConfig::default()
                .error_handler(|err, _req| ApiError::Validation(err.to_string()).into()),
        )
        .app_data(
            JsonConfig::default()
                .error_handler(|err, _req| ApiError::Validation(err.to_string()).into()),
        )
        .app_data(Data::new(solution_config.redis_pool))
        .app_data(Data::new(solution_config.postgres_pool))
        .configure(routes::config)
        .default_service(get().to(not_found));
    }
}
