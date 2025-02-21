use std::sync::Arc;

use crate::{cqrs::{CreateProductCommandHandler, GetProductsQueryHandler}, repositories::{InMemoryProductRepository}, uow::RepositoryContext};

#[derive(Clone)]
pub struct AppState {
    pub product_repository: Arc<InMemoryProductRepository>,
    pub uow: Arc<RepositoryContext<InMemoryProductRepository>>,
    pub create_product_command_handler: Arc<CreateProductCommandHandler<InMemoryProductRepository>>,
    pub get_products_query_handler: Arc<GetProductsQueryHandler<InMemoryProductRepository>>,
}