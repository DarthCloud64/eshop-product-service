use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};

use crate::uow::{ProductUnitOfWork, UnitOfWork};
use crate::{
    domain::Product,
    dtos::{CreateProductResponse, EmptyResponse, GetProductsResponse, ProductResponse, Response},
};

// traits
pub trait Command {}
pub trait Query {}

#[async_trait]
pub trait CommandHandler<C: Command, R: Response> {
    async fn handle(&self, input: &C) -> Result<R, String>;
}

#[async_trait]
pub trait QueryHandler<Q: Query, R: Response> {
    async fn handle(&self, input: Option<Q>) -> Result<R, String>;
}

// commands
#[derive(Serialize, Deserialize)]
pub struct CreateProductCommand {
    pub name: String,
    pub price: f32,
    pub description: String,
}
impl Command for CreateProductCommand {}

#[derive(Serialize, Deserialize)]
pub struct ModifyProductInventoryCommand {
    pub product_id: String,
    pub new_inventory: u32,
}
impl Command for ModifyProductInventoryCommand {}

#[derive(Serialize, Deserialize)]
pub struct DecrementProductReservedInventoryCommand {
    pub product_id: String,
}
impl Command for DecrementProductReservedInventoryCommand {}

pub struct IncrementProdcuctReservedInventoryCommand {
    pub product_id: String,
}
impl Command for IncrementProdcuctReservedInventoryCommand {}

// queries
pub struct GetProductsQuery {
    pub id: String,
}
impl Query for GetProductsQuery {}

// command handlers
#[derive(Clone)]
pub struct CreateProductCommandHandler {
    uow: Arc<dyn UnitOfWork + Send + Sync>,
}

impl CreateProductCommandHandler {
    pub fn new(uow: Arc<dyn UnitOfWork + Send + Sync>) -> Self {
        CreateProductCommandHandler { uow: uow }
    }
}

#[async_trait]
impl CommandHandler<CreateProductCommand, CreateProductResponse> for CreateProductCommandHandler {
    async fn handle(&self, input: &CreateProductCommand) -> Result<CreateProductResponse, String> {
        if input.price <= 0.0 {
            return Err(String::from("Price cannot be 0 or negative!!!"));
        }

        if input.name.is_empty() {
            return Err(String::from("Name cannot be empty!!!"));
        }

        if input.description.is_empty() {
            return Err(String::from("Description cannot be empty!!!"));
        }

        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("oops")
            .as_millis();

        let domain_product = Product {
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name.clone(),
            description: input.description.clone(),
            price: input.price,
            available_inventory: 0,
            reserved_inventory: 0,
            stars: 0,
            number_of_reviews: 0,
            created_at_utc: since_the_epoch as i64,
            updated_at_utc: since_the_epoch as i64,
            version: 0,
        };

        let product_repository = self.uow.get_product_repository().await;
        let session = self.uow.begin_transaction().await;

        match product_repository
            .create(domain_product.id.clone(), domain_product, session)
            .await
        {
            Ok(created_product) => match self.uow.commit().await {
                Ok(()) => Ok(CreateProductResponse {
                    id: created_product.id.clone(),
                }),
                Err(e) => {
                    event!(Level::WARN, "Error occurred while adding product: {}", e);
                    Err(e)
                }
            },
            Err(e) => {
                event!(Level::WARN, "Error occurred while adding product: {}", e);
                self.uow.rollback().await.unwrap();
                Err(e)
            }
        }
    }
}

// query handlers
#[derive(Clone)]
pub struct GetProductsQueryHandler {
    uow: Arc<dyn UnitOfWork + Send + Sync>,
}

impl GetProductsQueryHandler {
    pub fn new(uow: Arc<dyn UnitOfWork + Send + Sync>) -> Self {
        GetProductsQueryHandler { uow: uow }
    }
}

#[async_trait]
impl QueryHandler<GetProductsQuery, GetProductsResponse> for GetProductsQueryHandler {
    async fn handle(
        &self,
        input_option: Option<GetProductsQuery>,
    ) -> Result<GetProductsResponse, String> {
        let product_repository = self.uow.get_product_repository().await;
        match input_option {
            Some(input) => match product_repository.read(input.id.as_str()).await {
                Ok(domain_product) => {
                    let mut products = Vec::new();

                    products.push(ProductResponse {
                        id: domain_product.id.clone(),
                        name: domain_product.name.clone(),
                        price: domain_product.price,
                        description: domain_product.description.clone(),
                        available_inventory: domain_product.available_inventory,
                        reserved_inventory: domain_product.reserved_inventory,
                        stars: domain_product.stars,
                        number_of_reviews: domain_product.number_of_reviews,
                    });

                    Ok(GetProductsResponse { products: products })
                }
                Err(e) => {
                    event!(Level::WARN, "Error occurred while adding product: {}", e);
                    Err(e)
                }
            },
            None => match product_repository.read_all().await {
                Ok(domain_products) => {
                    let mut products = Vec::new();

                    for domain_product in domain_products {
                        products.push(ProductResponse {
                            id: domain_product.id.clone(),
                            name: domain_product.name.clone(),
                            price: domain_product.price,
                            description: domain_product.description.clone(),
                            available_inventory: domain_product.available_inventory,
                            reserved_inventory: domain_product.reserved_inventory,
                            stars: domain_product.stars,
                            number_of_reviews: domain_product.number_of_reviews,
                        });
                    }

                    Ok(GetProductsResponse { products: products })
                }
                Err(e) => {
                    event!(Level::WARN, "Error occurred while adding product: {}", e);
                    Err(e)
                }
            },
        }
    }
}

pub struct ModifyProductInventoryCommandHandler {
    uow: Arc<dyn UnitOfWork + Send + Sync>,
}

impl ModifyProductInventoryCommandHandler {
    pub fn new(uow: Arc<dyn UnitOfWork + Send + Sync>) -> Self {
        ModifyProductInventoryCommandHandler { uow: uow }
    }
}

#[async_trait]
impl CommandHandler<ModifyProductInventoryCommand, EmptyResponse>
    for ModifyProductInventoryCommandHandler
{
    async fn handle(&self, input: &ModifyProductInventoryCommand) -> Result<EmptyResponse, String> {
        let product_repository = self.uow.get_product_repository().await;

        match product_repository.read(&input.product_id).await {
            Ok(mut found_product) => {
                found_product.available_inventory = input.new_inventory;

                let session = self.uow.begin_transaction().await;

                match product_repository
                    .update(input.product_id.clone(), found_product, session)
                    .await
                {
                    Ok(_) => {
                        self.uow.commit().await.unwrap();
                        Ok(EmptyResponse {})
                    }
                    Err(e) => {
                        self.uow.rollback().await.unwrap();
                        Err(format!(
                            "Error occurred while modifying product inventory for product {}: {}",
                            &input.product_id, e
                        ))
                    }
                }
            }
            Err(e) => Err(format!(
                "Error occurred while modifying product inventory for product {}: {}",
                &input.product_id, e
            )),
        }
    }
}

pub struct DecrementProductInventoryCommandHandler {
    uow: Arc<dyn UnitOfWork + Send + Sync>,
}

impl DecrementProductInventoryCommandHandler {
    pub fn new(uow: Arc<dyn UnitOfWork + Send + Sync>) -> Self {
        DecrementProductInventoryCommandHandler { uow: uow }
    }
}

#[async_trait]
impl CommandHandler<DecrementProductReservedInventoryCommand, EmptyResponse>
    for DecrementProductInventoryCommandHandler
{
    async fn handle(
        &self,
        input: &DecrementProductReservedInventoryCommand,
    ) -> Result<EmptyResponse, String> {
        let product_repository = self.uow.get_product_repository().await;

        match product_repository.read(&input.product_id.as_str()).await {
            Ok(mut domain_product) => {
                domain_product.reserved_inventory = domain_product.reserved_inventory - 1;

                let session = self.uow.begin_transaction().await;

                match product_repository
                    .update(domain_product.id.clone(), domain_product, session)
                    .await
                {
                    Ok(_) => {
                        self.uow.commit().await.unwrap();
                        Ok(EmptyResponse {})
                    }
                    Err(e) => {
                        self.uow.rollback().await.unwrap();
                        event!(Level::WARN, "Error occurred while updating product while decrementing inventory for product {}: {}", input.product_id, e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                event!(
                    Level::WARN,
                    "Error occurred while decrementing product inventory for product {}: {}",
                    input.product_id,
                    e
                );
                Err(e)
            }
        }
    }
}

pub struct IncrementProdcuctInventoryCommandHandler {
    uow: Arc<dyn UnitOfWork + Send + Sync>,
}

impl IncrementProdcuctInventoryCommandHandler {
    pub fn new(uow: Arc<dyn UnitOfWork + Send + Sync>) -> Self {
        IncrementProdcuctInventoryCommandHandler { uow: uow }
    }
}

#[async_trait]
impl CommandHandler<IncrementProdcuctReservedInventoryCommand, EmptyResponse>
    for IncrementProdcuctInventoryCommandHandler
{
    async fn handle(
        &self,
        input: &IncrementProdcuctReservedInventoryCommand,
    ) -> Result<EmptyResponse, String> {
        let product_repository = self.uow.get_product_repository().await;

        match product_repository.read(&input.product_id.as_str()).await {
            Ok(mut domain_product) => {
                domain_product.reserved_inventory = domain_product.reserved_inventory + 1;

                let session = self.uow.begin_transaction().await;

                match product_repository
                    .update(domain_product.id.clone(), domain_product, session)
                    .await
                {
                    Ok(_) => {
                        self.uow.commit().await.unwrap();
                        Ok(EmptyResponse {})
                    }
                    Err(e) => {
                        self.uow.rollback().await.unwrap();
                        event!(Level::WARN, "Error occurred while updating product while incrementing inventory for product {}: {}", input.product_id, e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                event!(
                    Level::WARN,
                    "Error occurred while incrementing product inventory for product {}: {}",
                    input.product_id,
                    e
                );
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::uow::MockUnitOfWork;

    use super::*;

    #[tokio::test]
    async fn create_product_command_handler_returns_err_when_price_is_negative() {
        // Arrange
        let create_product_command: CreateProductCommand = CreateProductCommand {
            name: String::from("laptop"),
            price: -1.0,
            description: String::from("desc"),
        };

        let handler: CreateProductCommandHandler =
            CreateProductCommandHandler::new(Arc::new(MockUnitOfWork::new()));

        // Act
        let result = handler.handle(&create_product_command).await;

        // Assert
        assert!(result.is_err())
    }
}
