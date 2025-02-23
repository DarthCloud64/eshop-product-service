use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{domain::Product, dtos::{CreateProductResponse, GetProductsResponse, ProductResponse, Response}, events::MessageBroker, repositories::{InMemoryProductRepository, ProductRepository}, uow::{self, RepositoryContext}};

// traits
pub trait Command{}
pub trait Query{}

pub trait CommandHandler<C: Command, R: Response>{
    async fn handle(&self, input: &C) -> Result<R, String>;
}

pub trait QueryHandler<Q: Query, R: Response>{
    async fn handle(&self, input: &Q) -> Result<R, String>;
}

// commands
#[derive(Serialize, Deserialize)]
pub struct CreateProductCommand{
    pub name: String,
}
impl Command for CreateProductCommand{}

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
        let domain_product = Product{
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name.clone(),
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
                        println!("Error occurred while adding product: {}", e);
                        Err(e)
                    }
                }
            },
            Err(e) => {
                println!("Error occurred while adding product: {}", e);
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
    async fn handle(&self, input: &GetProductsQuery) -> Result<GetProductsResponse, String> {    
        match self.uow.product_repository.read(input.id.as_str()).await{
            Ok(domain_product) => {
                let mut products = Vec::new();

                products.push(ProductResponse{
                    id: domain_product.id.clone(),
                    name: domain_product.name.clone(),
                    inventory: domain_product.inventory,
                    stars: domain_product.stars,
                    number_of_reviews: domain_product.number_of_reviews
                });

                Ok(GetProductsResponse{
                    products: products
                })
            },
            Err(e) => {
                println!("Error occurred while adding product: {}", e);
                Err(e)
            }
        }
    }
}