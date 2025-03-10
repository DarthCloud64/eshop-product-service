use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f32,
    pub description: String,
    pub inventory: u32,
    pub stars: u8,
    pub number_of_reviews: u32,
}