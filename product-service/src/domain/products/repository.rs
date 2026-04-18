use async_trait::async_trait;

use crate::domain::products::entity::{Product, ProductInput};
use crate::domain::products::error::DomainError;

#[async_trait]
pub trait ProductRepository {
    async fn list_products(&self) -> Result<Vec<Product>, DomainError>;
    async fn get_product_by_id(&self, id: &str) -> Result<Product, DomainError>;
    async fn create_product(&self, input: ProductInput) -> Result<Product, DomainError>;
    async fn update_product(&self, id: &str, input: ProductInput) -> Result<Product, DomainError>;
    async fn delete_product(&self, id: &str) -> Result<Product, DomainError>;
}
