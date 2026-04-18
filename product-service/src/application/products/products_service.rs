use std::sync::Arc;

use crate::application::products::dto::{CreateProductDto, UpdateProductDto};
use crate::application::products::error::AppError;
use crate::domain::products::entity::Product;
use crate::domain::products::repository::ProductRepository;

type Result<T> = std::result::Result<T, AppError>;
type Repository = Arc<dyn ProductRepository + Send + Sync>;

pub struct ProductService {
    repository: Repository,
}

impl ProductService {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub async fn list_products(&self) -> Result<Vec<Product>> {
        self.repository
            .list_products()
            .await
            .map_err(AppError::from)
    }

    pub async fn get_product_by_id(&self, id: &str) -> Result<Product> {
        self.repository
            .get_product_by_id(id)
            .await
            .map_err(AppError::from)
    }

    pub async fn create_product(&self, dto: CreateProductDto) -> Result<Product> {
        self.repository
            .create_product(dto.into())
            .await
            .map_err(AppError::from)
    }

    pub async fn update_product(&self, id: &str, dto: UpdateProductDto) -> Result<Product> {
        self.repository
            .update_product(id, dto.into())
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_product(&self, id: &str) -> Result<Product> {
        self.repository
            .delete_product(id)
            .await
            .map_err(AppError::from)
    }
}
