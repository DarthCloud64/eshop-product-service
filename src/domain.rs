#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub inventory: u32,
    pub stars: u8,
    pub number_of_reviews: u32,
}