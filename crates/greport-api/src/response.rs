//! API response types

use serde::Serialize;

/// Standard API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn ok(data: T) -> Self {
        Self { data }
    }
}

/// Paginated response
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    /// Response data
    pub data: Vec<T>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

/// Pagination metadata
#[derive(Serialize)]
pub struct PaginationMeta {
    /// Current page
    pub page: u32,
    /// Items per page
    pub per_page: u32,
    /// Total items
    pub total: u32,
    /// Total pages
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    /// Create a paginated response
    pub fn new(data: Vec<T>, page: u32, per_page: u32, total: u32) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        Self {
            data,
            meta: PaginationMeta {
                page,
                per_page,
                total,
                total_pages,
            },
        }
    }
}
