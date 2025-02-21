use std::sync::{Arc, Mutex};
use axum::extract::{Json, Path, State};
use serde_json::{Value, json};

use crate::{cqrs::{CommandHandler, CreateProductCommand, CreateProductCommandHandler, GetProductsQuery, GetProductsQueryHandler, QueryHandler}, dtos::{CreateProductResponse, GetProductsResponse}, repositories::ProductRepository, state::AppState};

pub async fn index() -> &'static str {
    "Hello, World!"
}

pub async fn get_products(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Json<Value> {
    let input = GetProductsQuery {
        id: id.to_string()
    };

    let response = state.get_products_query_handler.handle(&input).await;

    Json(json!(response))
}

pub async fn create_product(state: State<Arc<AppState>>, Json(create_product_command): Json<CreateProductCommand>) -> Json<Value> {
    Json(json!(state.create_product_command_handler.handle(&create_product_command).await))
}