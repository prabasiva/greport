//! Database layer for greport
//!
//! Provides database models, queries, and migrations for caching
//! GitHub data and storing user configuration.

pub mod models;
pub mod queries;

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use thiserror::Error;

/// Database connection pool type
pub type DbPool = PgPool;

/// Database error type
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Database URL (postgres://...)
    pub database_url: String,
    /// Maximum connections in pool
    pub max_connections: u32,
    /// Connection acquire timeout in seconds
    pub acquire_timeout_secs: u64,
    /// Run migrations on connect
    pub run_migrations: bool,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            max_connections: 10,
            acquire_timeout_secs: 5,
            run_migrations: true,
        }
    }
}

impl DbConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self, DbError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| DbError::Config("DATABASE_URL not set".to_string()))?;

        Ok(Self {
            database_url,
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            acquire_timeout_secs: std::env::var("DB_ACQUIRE_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            run_migrations: std::env::var("DB_RUN_MIGRATIONS")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
        })
    }
}

/// Create a database connection pool with custom config
pub async fn create_pool_with_config(config: &DbConfig) -> Result<DbPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .connect(&config.database_url)
        .await?;

    if config.run_migrations {
        sqlx::migrate!("./migrations").run(&pool).await?;
    }

    Ok(pool)
}

/// Create a database connection pool from environment
pub async fn create_pool() -> Result<DbPool, DbError> {
    let config = DbConfig::from_env()?;
    create_pool_with_config(&config).await
}

/// Create a pool for testing (uses test database)
#[cfg(test)]
pub async fn create_test_pool() -> Result<DbPool, DbError> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/greport_test".to_string());

    let config = DbConfig {
        database_url,
        max_connections: 5,
        acquire_timeout_secs: 5,
        run_migrations: true,
    };

    create_pool_with_config(&config).await
}

/// Check database connectivity
pub async fn check_health(pool: &DbPool) -> Result<(), DbError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Re-export commonly used types
pub use models::*;
pub use queries::RepositoryStats;
