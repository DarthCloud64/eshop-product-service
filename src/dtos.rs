use serde::{Deserialize, Serialize};

pub trait Response{}

#[derive(Deserialize, Serialize)]
pub struct ProductResponse{
    pub id: String,
    pub name: String,
    pub price: f32,
    pub description: String,
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

#[derive(Deserialize, Serialize)]
pub struct ApiError{
    pub error: String
}
impl Response for ApiError{}

#[derive(Deserialize, Serialize)]
pub struct EmptyResponse{}
impl Response for EmptyResponse{}