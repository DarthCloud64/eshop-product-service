use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{event, Level};

use crate::{domain::Product, dtos::{CreateProductResponse, EmptyResponse, GetProductsResponse, ProductResponse, Response}, events::MessageBroker, repositories::ProductRepository, uow::RepositoryContext};

// traits
pub trait Command{}
pub trait Query{}

pub trait CommandHandler<C: Command, R: Response>{
    async fn handle(&self, input: &C) -> Result<R, String>;
}

pub trait QueryHandler<Q: Query, R: Response>{
    async fn handle(&self, input: Option<Q>) -> Result<R, String>;
}

// commands
#[derive(Serialize, Deserialize)]
pub struct CreateProductCommand{
    pub name: String,
    pub price: f32,
    pub description: String,
}
impl Command for CreateProductCommand{}

#[derive(Serialize, Deserialize)]
pub struct ModifyProductInventoryCommand{
    pub product_id: String,
    pub new_inventory: u32
}
impl Command for ModifyProductInventoryCommand{}

// queries
pub struct GetProductsQuery{
    pub id: String
}
impl Query for GetProductsQuery{}

// command handlers
#[derive(Clone)]
pub struct CreateProductCommandHandler<T1: ProductRepository, T2: MessageBroker>{
    uow: Arc<RepositoryContext<T1, T2>>
}

impl<T1: ProductRepository, T2: MessageBroker> CreateProductCommandHandler<T1, T2>{
    pub fn new(uow: Arc<RepositoryContext<T1, T2>>) -> Self{
        CreateProductCommandHandler{
            uow: uow
        }
    }
}

impl<T1: ProductRepository, T2: MessageBroker> CommandHandler<CreateProductCommand, CreateProductResponse> for CreateProductCommandHandler<T1, T2>{
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

        let domain_product = Product{
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name.clone(),
            description: input.description.clone(),
            price: input.price,
            inventory: 0,
            stars: 0,
            number_of_reviews: 0
        };

        match self.uow.add_product(domain_product.id.clone(), domain_product).await{
            Ok(created_product) => {
                match self.uow.commit().await {
                    Ok(()) => Ok(CreateProductResponse {
                        id: created_product.id.clone()
                    }),
                    Err(e) => {
                        event!(Level::WARN, "Error occurred while adding product: {}", e);
                        Err(e)
                    }
                }
            },
            Err(e) => {
                event!(Level::WARN, "Error occurred while adding product: {}", e);
                Err(e)
            }
        }
    }
}

// query handlers
#[derive(Clone)]
pub struct GetProductsQueryHandler<T1: ProductRepository, T2: MessageBroker>{
    uow: Arc<RepositoryContext<T1, T2>>
}

impl<T1: ProductRepository, T2: MessageBroker> GetProductsQueryHandler<T1, T2>{
    pub fn new(uow: Arc<RepositoryContext<T1, T2>>) -> Self {
        GetProductsQueryHandler{
            uow: uow
        }
    }
}

impl<T1: ProductRepository, T2: MessageBroker> QueryHandler<GetProductsQuery, GetProductsResponse> for GetProductsQueryHandler<T1, T2>{
    async fn handle(&self, input_option: Option<GetProductsQuery>) -> Result<GetProductsResponse, String> {    
        match input_option {
            Some(input) => {
                match self.uow.product_repository.read(input.id.as_str()).await{
                    Ok(domain_product) => {
                        let mut products = Vec::new();
        
                        products.push(ProductResponse{
                            id: domain_product.id.clone(),
                            name: domain_product.name.clone(),
                            price: domain_product.price,
                            description: domain_product.description.clone(),
                            inventory: domain_product.inventory,
                            stars: domain_product.stars,
                            number_of_reviews: domain_product.number_of_reviews
                        });
        
                        Ok(GetProductsResponse{
                            products: products
                        })
                    },
                    Err(e) => {
                        event!(Level::WARN, "Error occurred while adding product: {}", e);
                        Err(e)
                    }
                }
            },
            None => {
                match self.uow.product_repository.read_all().await{
                    Ok(domain_products) => {
                        let mut products = Vec::new();

                        for domain_product in domain_products {
                            products.push(ProductResponse{
                                id: domain_product.id.clone(),
                                name: domain_product.name.clone(),
                                price: domain_product.price,
                                description: domain_product.description.clone(),
                                inventory: domain_product.inventory,
                                stars: domain_product.stars,
                                number_of_reviews: domain_product.number_of_reviews
                            });
                        }
                        
                        Ok(GetProductsResponse{
                            products: products
                        })
                    },
                    Err(e) => {
                        event!(Level::WARN, "Error occurred while adding product: {}", e);
                        Err(e)
                    }
                }
            }
        }
    }
}

pub struct ModifyProductInventoryCommandHandler<T1: ProductRepository, T2: MessageBroker>{
    uow: Arc<RepositoryContext<T1, T2>>
}

impl <T1: ProductRepository, T2: MessageBroker> ModifyProductInventoryCommandHandler<T1, T2>{
    pub fn new(uow: Arc<RepositoryContext<T1, T2>>) -> Self {
        ModifyProductInventoryCommandHandler {
            uow: uow
        }
    }
}

impl <T1: ProductRepository, T2: MessageBroker> CommandHandler<ModifyProductInventoryCommand, EmptyResponse> for ModifyProductInventoryCommandHandler<T1, T2>{
    async fn handle(&self, input: &ModifyProductInventoryCommand) -> Result<EmptyResponse, String> {
        match self.uow.product_repository.read(&input.product_id).await {
            Ok(mut found_product) => {
                found_product.inventory = input.new_inventory;

                match self.uow.product_repository.update(input.product_id.clone(), found_product).await{
                    Ok(_) => {
                        Ok(EmptyResponse{})
                    },
                    Err(e) => {
                        Err(format!("Error occurred while modifying product inventory for product {}: {}", &input.product_id, e))
                    }
                }
            },
            Err(e) => {
                Err(format!("Error occurred while modifying product inventory for product {}: {}", &input.product_id, e))
            }
        }
    }
}