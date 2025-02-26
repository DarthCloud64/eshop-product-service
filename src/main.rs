// define modules in crate
mod routes;
mod dtos;
mod repositories;
mod domain;
mod uow;
mod cqrs;
mod state;
mod events;

use std::sync::Arc;
use axum::{handler::Handler, routing::{get, post}, Router};
use cqrs::{CreateProductCommandHandler, GetProductsQueryHandler};
use domain::Product;
use events::{RabbitMqInitializationInfo, RabbitMqMessageBroker};
use repositories::{InMemoryProductRepository, MongoDbInitializationInfo, MongoDbProductRepository};
use routes::*;
use state::AppState;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uow::RepositoryContext;

#[tokio::main]
async fn main() {
    let info = MongoDbInitializationInfo{
        uri: String::from("mongodb://localhost:27017"),
        database: String::from("eshop-product"),
        collection: String::from("products")
    };

    // let product_repository= Arc::new(InMemoryProductRepository::new());
    let product_repository = Arc::new(MongoDbProductRepository::new(&info).await);

    let message_broker = Arc::new(RabbitMqMessageBroker::new(RabbitMqInitializationInfo::new(String::from("localhost"), 5672, String::from("guest"), String::from("guest"))).await.unwrap());
    let uow = Arc::new(RepositoryContext::new(product_repository.clone(), message_broker.clone()));
    let create_product_command_handler = Arc::new(CreateProductCommandHandler::new(uow.clone()));
    let get_products_query_handler = Arc::new(GetProductsQueryHandler::new(uow.clone()));

    let state = Arc::new(AppState{
        product_repository: product_repository,
        uow: uow,
        create_product_command_handler: create_product_command_handler,
        get_products_query_handler: get_products_query_handler,
    });
    
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9090").await.unwrap();

    axum::serve(listener, Router::new()
        .route("/", get(index))
        .route("/products/{id}", get(get_products))
        .route("/products", post(create_product))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::new())
        )).await.unwrap();
}
