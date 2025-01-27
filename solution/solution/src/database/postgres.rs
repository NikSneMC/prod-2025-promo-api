use std::time::Duration;

use log::info;
use sqlx::{
    migrate, migrate::MigrateDatabase, postgres::PgPoolOptions, Connection, PgConnection, PgPool,
    Postgres,
};

use crate::POSTGRES_CONN;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    info!("Initializing database connection");
    let database_url = POSTGRES_CONN();
    let pool = PgPoolOptions::new()
        .min_connections(0)
        .max_connections(16)
        .max_lifetime(Some(Duration::from_secs(60 * 60)))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn check_for_migrations() -> Result<(), sqlx::Error> {
    let uri = POSTGRES_CONN();
    if !Postgres::database_exists(uri).await? {
        info!("Creating database...");
        Postgres::create_database(uri).await?;
    }

    info!("Applying migrations...");

    let mut conn: PgConnection = PgConnection::connect(uri).await?;
    migrate!()
        .run(&mut conn)
        .await
        .expect("Error while running database migrations!");

    Ok(())
}
