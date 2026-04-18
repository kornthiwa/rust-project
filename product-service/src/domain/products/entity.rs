use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct ProductInput {
    pub active: Option<bool>,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Product {
    pub id: i32,
    pub active: bool,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
