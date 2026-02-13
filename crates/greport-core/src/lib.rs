//! greport-core - Core library for GitHub reporting and analytics
//!
//! This crate provides the core functionality for the greport tool:
//! - GitHub API client abstraction
//! - Data models for issues, pull requests, releases
//! - Metrics calculations (velocity, SLA, burndown)
//! - Report generation

pub mod client;
pub mod config;
pub mod error;
pub mod metrics;
pub mod models;
pub mod reports;

pub use client::{
    GitHubClient, GitHubClientRegistry, OctocrabClient, OrgEntry, ProjectClient, RepoId,
};
pub use config::{Config, OrgConfig};
pub use error::{Error, Result};
