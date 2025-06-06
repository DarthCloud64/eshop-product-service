use std::sync::Arc;

use crate::cqrs::{
    CreateProductCommandHandler, DecrementProductInventoryCommandHandler, GetProductsQueryHandler,
    IncrementProdcuctInventoryCommandHandler, ModifyProductInventoryCommandHandler,
};

#[derive(Clone)]
pub struct AppState {
    pub create_product_command_handler: Arc<CreateProductCommandHandler>,
    pub get_products_query_handler: Arc<GetProductsQueryHandler>,
    pub modify_product_inventory_command_handler: Arc<ModifyProductInventoryCommandHandler>,
    pub decrement_product_inventory_command_handler: Arc<DecrementProductInventoryCommandHandler>,
    pub increment_product_inventory_command_handler: Arc<IncrementProdcuctInventoryCommandHandler>,
    pub auth0_domain: String,
    pub auth0_audience: String,
}
