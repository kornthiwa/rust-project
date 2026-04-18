use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};

use crate::app::AppState;
use crate::application::products::dto::{CreateProductDto, UpdateProductDto};
use crate::application::products::error::AppError;
use crate::domain::products::entity::Product;

pub async fn list_products_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Product>>, AppError> {
    let products = state.product_service.list_products().await?;
    Ok(Json(products))
}

pub async fn get_product_by_id_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Product>, AppError> {
    let product = state.product_service.get_product_by_id(&id).await?;
    Ok(Json(product))
}

pub async fn create_product_handler(
    State(state): State<Arc<AppState>>,
    dto: Result<Json<CreateProductDto>, JsonRejection>,
) -> Result<Json<Product>, AppError> {
    let Json(dto) = dto.map_err(AppError::from)?;
    let product = state.product_service.create_product(dto).await?;
    Ok(Json(product))
}

pub async fn update_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    dto: Result<Json<UpdateProductDto>, JsonRejection>,
) -> Result<Json<Product>, AppError> {
    let Json(dto) = dto.map_err(AppError::from)?;
    let product = state.product_service.update_product(&id, dto).await?;
    Ok(Json(product))
}

pub async fn delete_product_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Product>, AppError> {
    let product = state.product_service.delete_product(&id).await?;
    Ok(Json(product))
}
