use std::sync::Arc;

use crate::{cqrs::{CreateProductCommandHandler, GetProductsQueryHandler, ModifyProductInventoryCommandHandler}, events::RabbitMqMessageBroker, repositories::MongoDbProductRepository};

#[derive(Clone)]
pub struct AppState {
    pub create_product_command_handler: Arc<CreateProductCommandHandler<MongoDbProductRepository, RabbitMqMessageBroker>>,
    pub get_products_query_handler: Arc<GetProductsQueryHandler<MongoDbProductRepository, RabbitMqMessageBroker>>,
    pub modify_product_inventory_command_handler: Arc<ModifyProductInventoryCommandHandler<MongoDbProductRepository, RabbitMqMessageBroker>>,
    pub auth0_domain: String,
    pub auth0_audience: String,
}