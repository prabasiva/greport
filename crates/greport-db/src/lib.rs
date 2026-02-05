//! Database layer for greport
//!
//! Provides database models, queries, and migrations for caching
//! GitHub data and storing user configuration.

pub mod models;
pub mod queries;

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Database connection pool type
pub type DbPool = PgPool;

/// Create a database connection pool
pub async fn create_pool() -> anyhow::Result<DbPool> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Create a pool for testing (uses test database)
#[cfg(test)]
pub async fn create_test_pool() -> anyhow::Result<DbPool> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/greport_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
