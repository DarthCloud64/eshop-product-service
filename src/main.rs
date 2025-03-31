// define modules in crate
mod routes;
mod dtos;
mod repositories;
mod domain;
mod uow;
mod cqrs;
mod state;
mod events;
mod auth;
mod metrics;

use std::sync::Arc;
use axum::{middleware::from_fn_with_state, routing::{get, post, put}, Router};
use axum_prometheus::PrometheusMetricLayer;
use cqrs::{CreateProductCommandHandler, GetProductsQueryHandler, ModifyProductInventoryCommandHandler};
use events::{MessageBroker, RabbitMqInitializationInfo, RabbitMqMessageBroker};
use repositories::{MongoDbInitializationInfo, MongoDbProductRepository};
use routes::*;
use state::AppState;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uow::RepositoryContext;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let info = MongoDbInitializationInfo{
        uri: String::from(env::var("MONGODB_URI").unwrap()),
        database: String::from(env::var("MONGODB_DB").unwrap()),
        collection: String::from(env::var("MONGODB_COLLECTION").unwrap())
    };

    // let product_repository= Arc::new(InMemoryProductRepository::new());
    let product_repository = Arc::new(MongoDbProductRepository::new(&info).await);

    let message_broker = Arc::new(RabbitMqMessageBroker::new(RabbitMqInitializationInfo::new(String::from(env::var("RABBITMQ_URI").unwrap()), env::var("RABBITMQ_PORT").unwrap().parse().unwrap(), String::from(env::var("RABBITMQ_USER").unwrap()), String::from(env::var("RABBITMQ_PASS").unwrap()))).await.unwrap());
    let uow = Arc::new(RepositoryContext::new(product_repository.clone(), message_broker.clone()));
    let create_product_command_handler = Arc::new(CreateProductCommandHandler::new(uow.clone()));
    let get_products_query_handler = Arc::new(GetProductsQueryHandler::new(uow.clone()));
    let modify_product_inventory_command_handler = Arc::new(ModifyProductInventoryCommandHandler::new(uow.clone()));

    let state = Arc::new(AppState{
        create_product_command_handler: create_product_command_handler,
        get_products_query_handler: get_products_query_handler,
        modify_product_inventory_command_handler: modify_product_inventory_command_handler,
        auth0_domain: String::from(env::var("AUTH0_DOMAIN").unwrap()),
        auth0_audience: String::from(env::var("AUTH0_AUDIENCE").unwrap()),
    });
    
    tracing_subscriber::
        fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_ansi(false)
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_current_span(true)
        .with_writer(std::fs::File::create(String::from(env::var("LOG_PATH").unwrap())).unwrap())
        .init();

    let (prometheus_layer, metrics_handle) = PrometheusMetricLayer::pair();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", env::var("AXUM_PORT").unwrap())).await.unwrap();

    tokio::spawn(async move {
        message_broker.consume("product.added.to.cart").await;
    });

    axum::serve(listener, Router::new()
        .route("/", get(index))

        .route("/metrics", get(|| async move {metrics_handle.render()}))

        .route("/products/{id}", 
            get(get_products)
            .route_layer(from_fn_with_state(state.clone(), auth::authentication_middleware)))

        .route("/products", 
            post(create_product)
            .get(get_all_products)
            .route_layer(from_fn_with_state(state.clone(), auth::authentication_middleware)))

        .route("/products/modifyProductInventory", 
            put(modify_product_inventory)
            .route_layer(from_fn_with_state(state.clone(), auth::authentication_middleware)))
            
        .with_state(state)
        .layer(prometheus_layer)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )).await.unwrap();
}
