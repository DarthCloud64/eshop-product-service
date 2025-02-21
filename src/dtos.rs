use serde::{Deserialize, Serialize};

pub trait Response{}

#[derive(Deserialize, Serialize)]
pub struct ProductResponse{
    pub id: String,
    pub name: String,
    pub inventory: u32,
    pub stars: u8,
    pub number_of_reviews: u32,
}

#[derive(Deserialize, Serialize)]
pub struct GetProductsResponse{
    pub products: Vec<ProductResponse>
}
impl Response for GetProductsResponse{}

#[derive(Deserialize, Serialize)]
pub struct CreateProductResponse{
    pub id: String
}
impl Response for CreateProductResponse{}

