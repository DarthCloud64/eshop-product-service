use tokio::sync::Mutex;

use crate::domain::Product;
use std::{collections::HashMap, sync::Arc};

pub trait ProductRepository {
    async fn create(&self, id: String, product: Product) -> Result<Product, String>;
    async fn read<'a>(&self, id: &'a str) -> Result<Product, String>;
    async fn update(&self, id: String, product: Product) -> Result<Product, String>;
    async fn delete(&self, id: &str);
}

#[derive(Clone)]
pub struct InMemoryProductRepository {
    products: Arc<Mutex<HashMap<String, Product>>>,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        InMemoryProductRepository {
            products: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ProductRepository for InMemoryProductRepository {
    async fn create(&self, id: String, product: Product) -> Result<Product, String> {
        let mut lock = self.products.lock().await;
        lock.insert(id.clone(), product.clone());
        match  lock.get(id.as_str()) {
            Some(x) => {
                Ok(x.clone())
            },
            None => {
                Err(format!("Product with id {} did not exist", id))
            }
        }
    }

    async fn read<'a>(&self, id: &'a str) -> Result<Product, String> {
        let lock = self.products.lock().await;
        match  lock.get(id) {
            Some(x) => {
                Ok(x.clone())
            },
            None => {
                Err(format!("Product with id {} did not exist", id))
            }
        }
    }

    async fn update(&self, id: String, product: Product) -> Result<Product, String> {
        let mut lock = self.products.lock().await;
        lock.insert(id.clone(), product.clone());
        match lock.get(id.as_str()){
            Some(x) => {
                Ok(x.clone())
            },
            None => {
                Err(format!("Product with id {} did not exist", id))
            }
        }
    }

    async fn delete(&self, id: &str) {
        let mut lock = self.products.lock().await;
        lock.remove_entry(id);
    }
}