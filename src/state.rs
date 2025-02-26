use std::sync::Arc;

use crate::{cqrs::{CreateProductCommandHandler, GetProductsQueryHandler}, events::RabbitMqMessageBroker, repositories::{InMemoryProductRepository, MongoDbProductRepository}, uow::RepositoryContext};

#[derive(Clone)]
pub struct AppState {
    pub product_repository: Arc<MongoDbProductRepository>,
    pub uow: Arc<RepositoryContext<MongoDbProductRepository, RabbitMqMessageBroker>>,
    pub create_product_command_handler: Arc<CreateProductCommandHandler<MongoDbProductRepository, RabbitMqMessageBroker>>,
    pub get_products_query_handler: Arc<GetProductsQueryHandler<MongoDbProductRepository, RabbitMqMessageBroker>>,
}