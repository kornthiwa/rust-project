use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jiff::Timestamp;
use tokio::sync::Mutex;

use crate::domain::products::entity::{Product as DomainProduct, ProductInput};
use crate::domain::products::error::DomainError;
use crate::domain::products::repository::ProductRepository;

#[derive(Debug, toasty::Model)]
pub struct Product {
    #[key]
    #[auto]
    pub id: i32,
    pub active: bool,
    #[unique]
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub deleted_at: Option<Timestamp>,
}

pub struct ToastyProductRepository {
    db: Arc<Mutex<toasty::Db>>,
}

impl ToastyProductRepository {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }
}

fn ts_to_utc(ts: Option<Timestamp>) -> Result<DateTime<Utc>, DomainError> {
    let parsed = ts.unwrap_or_else(Timestamp::now).to_string();
    DateTime::parse_from_rfc3339(&parsed)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| DomainError::RepositoryFailure)
}

impl TryFrom<Product> for DomainProduct {
    type Error = DomainError;

    fn try_from(row: Product) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            active: row.active,
            sku: row.sku,
            name: row.name,
            description: row.description,
            price_cents: row.price_cents,
            created_at: ts_to_utc(row.created_at)?,
            updated_at: ts_to_utc(row.updated_at)?,
        })
    }
}

#[async_trait]
impl ProductRepository for ToastyProductRepository {
    async fn list_products(&self) -> Result<Vec<DomainProduct>, DomainError> {
        let mut db = self.db.lock().await;
        let rows = Product::all()
            .exec(&mut *db)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        rows.into_iter()
            .filter(|row| row.active && row.deleted_at.is_none())
            .map(DomainProduct::try_from)
            .collect()
    }

    async fn get_product_by_id(&self, id: &str) -> Result<DomainProduct, DomainError> {
        let product_id = id.parse::<i32>().map_err(|_| DomainError::RepositoryFailure)?;
        let mut db = self.db.lock().await;
        let row = Product::get_by_id(&mut *db, &product_id)
            .await
            .map_err(|_| DomainError::NotFound)?;

        if !row.active || row.deleted_at.is_some() {
            return Err(DomainError::NotFound);
        }

        DomainProduct::try_from(row)
    }

    async fn create_product(&self, input: ProductInput) -> Result<DomainProduct, DomainError> {
        let mut db = self.db.lock().await;
        let now = Timestamp::now();
        let created = toasty::create!(Product {
            active: input.active.unwrap_or(true),
            sku: input.sku,
            name: input.name,
            description: input.description,
            price_cents: input.price_cents,
            created_at: Some(now),
            updated_at: Some(now),
            deleted_at: Option::<Timestamp>::None,
        })
        .exec(&mut *db)
        .await
        .map_err(|_| DomainError::RepositoryFailure)?;

        DomainProduct::try_from(created)
    }

    async fn update_product(&self, id: &str, input: ProductInput) -> Result<DomainProduct, DomainError> {
        let product_id = id.parse::<i32>().map_err(|_| DomainError::RepositoryFailure)?;
        let mut db = self.db.lock().await;
        let now = Timestamp::now();
        let mut product = Product::get_by_id(&mut *db, &product_id)
            .await
            .map_err(|_| DomainError::NotFound)?;

        if product.deleted_at.is_some() {
            return Err(DomainError::NotFound);
        }

        let next_active = input.active.unwrap_or(product.active);

        product
            .update()
            .active(next_active)
            .sku(input.sku)
            .name(input.name)
            .description(input.description)
            .price_cents(input.price_cents)
            .updated_at(Some(now))
            .exec(&mut *db)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        let refreshed = Product::get_by_id(&mut *db, &product_id)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        DomainProduct::try_from(refreshed)
    }

    async fn delete_product(&self, id: &str) -> Result<DomainProduct, DomainError> {
        let product_id = id.parse::<i32>().map_err(|_| DomainError::RepositoryFailure)?;
        let mut db = self.db.lock().await;
        let now = Timestamp::now();
        let mut product = Product::get_by_id(&mut *db, &product_id)
            .await
            .map_err(|_| DomainError::NotFound)?;

        if product.deleted_at.is_some() {
            return Err(DomainError::NotFound);
        }

        product
            .update()
            .active(false)
            .updated_at(Some(now))
            .deleted_at(Some(now))
            .exec(&mut *db)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        let refreshed = Product::get_by_id(&mut *db, &product_id)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        DomainProduct::try_from(refreshed)
    }
}
