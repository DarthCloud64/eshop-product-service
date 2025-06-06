// define modules in crate
mod auth;
mod cqrs;
mod domain;
mod dtos;
mod events;
mod metrics;
mod repositories;
mod routes;
mod state;
mod uow;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put},
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use cqrs::{
    CreateProductCommandHandler, DecrementProductInventoryCommandHandler, GetProductsQueryHandler,
    IncrementProdcuctInventoryCommandHandler, ModifyProductInventoryCommandHandler,
};
use dotenv::dotenv;
use events::{MessageBroker, RabbitMqInitializationInfo, RabbitMqMessageBroker};
use mongodb::Client;
use repositories::{MongoDbInitializationInfo, MongoDbProductRepository};
use routes::*;
use state::AppState;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::uow::ProductUnitOfWork;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let info = MongoDbInitializationInfo {
        uri: String::from(env::var("MONGODB_URI").unwrap()),
        database: String::from(env::var("MONGODB_DB").unwrap()),
        collection: String::from(env::var("MONGODB_COLLECTION").unwrap()),
    };

    let client: Client = Client::with_uri_str(&info.uri).await.unwrap();

    let product_repository = Arc::new(MongoDbProductRepository::new(&info, &client).await);

    let client_session = Arc::new(Mutex::new(client.start_session().await.unwrap()));

    let message_broker = Arc::new(
        RabbitMqMessageBroker::new(RabbitMqInitializationInfo::new(
            String::from(env::var("RABBITMQ_URI").unwrap()),
            env::var("RABBITMQ_PORT").unwrap().parse().unwrap(),
            String::from(env::var("RABBITMQ_USER").unwrap()),
            String::from(env::var("RABBITMQ_PASS").unwrap()),
        ))
        .await
        .unwrap(),
    );
    let uow = Arc::new(ProductUnitOfWork::new(
        product_repository.clone(),
        message_broker.clone(),
        client_session,
    ));
    let create_product_command_handler = Arc::new(CreateProductCommandHandler::new(uow.clone()));
    let get_products_query_handler = Arc::new(GetProductsQueryHandler::new(uow.clone()));
    let modify_product_inventory_command_handler =
        Arc::new(ModifyProductInventoryCommandHandler::new(uow.clone()));
    let decrement_product_inventory_command_handler =
        Arc::new(DecrementProductInventoryCommandHandler::new(uow.clone()));
    let increment_product_inventory_command_handler =
        Arc::new(IncrementProdcuctInventoryCommandHandler::new(uow.clone()));

    let state = Arc::new(AppState {
        create_product_command_handler: create_product_command_handler,
        get_products_query_handler: get_products_query_handler,
        modify_product_inventory_command_handler: modify_product_inventory_command_handler,
        decrement_product_inventory_command_handler: decrement_product_inventory_command_handler,
        increment_product_inventory_command_handler: increment_product_inventory_command_handler,
        auth0_domain: String::from(env::var("AUTH0_DOMAIN").unwrap()),
        auth0_audience: String::from(env::var("AUTH0_AUDIENCE").unwrap()),
    });

    tracing_subscriber::fmt()
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

    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", env::var("AXUM_PORT").unwrap()))
            .await
            .unwrap();

    let state_clone_for_background_jobs = state.clone();
    let message_broke_for_background_jobs = message_broker.clone();
    tokio::spawn(async move {
        message_broke_for_background_jobs
            .consume(
                events::PRODUCT_ADDED_TO_CART_QUEUE_NAME,
                state_clone_for_background_jobs,
            )
            .await;
    });

    let state_clone_2 = state.clone();
    let message_broker_clone_2 = message_broker.clone();
    tokio::spawn(async move {
        message_broker_clone_2
            .consume(events::PRODUCT_REMOVED_FROM_CART_QUEUE_NAME, state_clone_2)
            .await;
    });

    axum::serve(
        listener,
        Router::new()
            .route("/", get(index))
            .route("/metrics", get(|| async move { metrics_handle.render() }))
            .route(
                "/products/{id}",
                get(get_products).route_layer(from_fn_with_state(
                    state.clone(),
                    auth::authentication_middleware,
                )),
            )
            .route(
                "/products",
                post(create_product)
                    .get(get_all_products)
                    .route_layer(from_fn_with_state(
                        state.clone(),
                        auth::authentication_middleware,
                    )),
            )
            .route(
                "/products/modifyProductInventory",
                put(modify_product_inventory).route_layer(from_fn_with_state(
                    state.clone(),
                    auth::authentication_middleware,
                )),
            )
            .with_state(state)
            .layer(prometheus_layer)
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive()),
            ),
    )
    .await
    .unwrap();
}
