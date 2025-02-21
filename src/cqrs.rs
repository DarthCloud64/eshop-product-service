use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{domain::Product, dtos::{CreateProductResponse, GetProductsResponse, ProductResponse, Response}, repositories::{InMemoryProductRepository, ProductRepository}, uow::{self, RepositoryContext}};

// traits
pub trait Command{}
pub trait Query{}

pub trait CommandHandler<C: Command, R: Response>{
    async fn handle(&self, input: &C) -> R;
}

pub trait QueryHandler<Q: Query, R: Response>{
    async fn handle(&self, input: &Q) -> R;
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
pub struct CreateProductCommandHandler<T: ProductRepository>{
    uow: Arc<RepositoryContext<T>>
}

impl<T: ProductRepository> CreateProductCommandHandler<T>{
    pub fn new(uow: Arc<RepositoryContext<T>>) -> Self{
        CreateProductCommandHandler{
            uow: uow
        }
    }
}

impl<T: ProductRepository> CommandHandler<CreateProductCommand, CreateProductResponse> for CreateProductCommandHandler<T>{
    async fn handle(&self, input: &CreateProductCommand) -> CreateProductResponse {
        let domain_product = Product{
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name.clone(),
            inventory: 0,
            stars: 0,
            number_of_reviews: 0
        };

        match self.uow.add_product(domain_product.id.clone(), domain_product).await{
            Ok(created_product) => CreateProductResponse {
                id: created_product.id.clone()
            },
            Err(e) => {
                println!("Error occurred while adding product: {}", e);
                panic!()
            }
        }
    }
}

// query handlers
#[derive(Clone)]
pub struct GetProductsQueryHandler<T: ProductRepository>{
    uow: Arc<RepositoryContext<T>>
}

impl<T: ProductRepository> GetProductsQueryHandler<T>{
    pub fn new(uow: Arc<RepositoryContext<T>>) -> Self {
        GetProductsQueryHandler{
            uow: uow
        }
    }
}

impl<T: ProductRepository> QueryHandler<GetProductsQuery, GetProductsResponse> for GetProductsQueryHandler<T>{
    async fn handle(&self, input: &GetProductsQuery) -> GetProductsResponse {    
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

                GetProductsResponse{
                    products: products
                }
            },
            Err(e) => {
                panic!()
            }
        }
    }
}