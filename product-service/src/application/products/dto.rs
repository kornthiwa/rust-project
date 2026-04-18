use serde::Deserialize;

use crate::domain::products::entity::ProductInput;

#[derive(Deserialize)]
pub struct CreateProductDto {
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
}

#[derive(Deserialize)]
pub struct UpdateProductDto {
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
}

impl From<CreateProductDto> for ProductInput {
    fn from(dto: CreateProductDto) -> Self {
        Self {
            active: Some(true),
            sku: dto.sku,
            name: dto.name,
            description: dto.description,
            price_cents: dto.price_cents,
        }
    }
}

impl From<UpdateProductDto> for ProductInput {
    fn from(dto: UpdateProductDto) -> Self {
        Self {
            active: None,
            sku: dto.sku,
            name: dto.name,
            description: dto.description,
            price_cents: dto.price_cents,
        }
    }
}
