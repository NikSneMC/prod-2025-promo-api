use actix_web::{middleware::Logger, App, HttpServer};
use actix_web_lab::middleware::CatchPanic;
use env_logger::Env;
use solution::database::redis::RedisPool;
use solution::{app_setup, database, solution_config};
use std::io::Result;

#[actix_rt::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    database::check_for_migrations()
        .await
        .expect("An error occurred while running migrations.");

    let pool = database::connect()
        .await
        .expect("Database connection failed");

    let redis_pool = RedisPool::new(None);

    let config = app_setup(pool, redis_pool);

    HttpServer::new(move || {
        App::new()
            .wrap(CatchPanic::default())
            .wrap(Logger::new("%a \"%r\" %s %b (took %D ms to serve)"))
            .configure(solution_config(config.clone()))
    })
    .bind(solution::SERVER_ADDRESS())?
    .run()
    .await
}
