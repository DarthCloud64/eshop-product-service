use std::sync::{Arc, Mutex};
use axum::{extract::{Json, Path, State}, http::StatusCode};
use serde_json::{Value, json};

use crate::{cqrs::{CommandHandler, CreateProductCommand, CreateProductCommandHandler, GetProductsQuery, GetProductsQueryHandler, QueryHandler}, dtos::{ApiError, CreateProductResponse, GetProductsResponse}, repositories::ProductRepository, state::AppState};

pub async fn index() -> &'static str {
    "Hello, World!"
}

pub async fn get_products(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    let input = GetProductsQuery {
        id: id.to_string()
    };

    match state.get_products_query_handler.handle(Some(input)).await {
        Ok(response)=> (StatusCode::OK, Json(json!(response))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ApiError{error: e})))
    }
}

pub async fn get_all_products(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    match state.get_products_query_handler.handle(None).await {
        Ok(response)=> (StatusCode::OK, Json(json!(response))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ApiError{error: e})))
    }
}

pub async fn create_product(state: State<Arc<AppState>>, Json(create_product_command): Json<CreateProductCommand>) -> (StatusCode, Json<Value>) {
    match state.create_product_command_handler.handle(&create_product_command).await {
        Ok(response) => (StatusCode::CREATED, Json(json!(response))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(ApiError{error: e})))
    }   
}