use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f32,
    pub description: String,
    pub available_inventory: u32,
    pub reserved_inventory: u32,
    pub stars: u8,
    pub number_of_reviews: u32,
    pub created_at_utc: i64,
    pub updated_at_utc: i64,
    pub version: u32,
}
