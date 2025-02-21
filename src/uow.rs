use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{domain::Product, repositories::ProductRepository};

#[derive(Clone)]
pub struct RepositoryContext<T: ProductRepository> {
    pub product_repository: Arc<T>,
    new_products: Arc<Mutex<HashMap<String, Product>>>
}

impl<T: ProductRepository> RepositoryContext<T> {
    pub fn new(product_repository: Arc<T>) -> RepositoryContext<T>{
        RepositoryContext {
            product_repository: product_repository,
            new_products: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn add_product(&self, id: String, product: Product) -> Result<Product, String>{
        let mut lock = self.new_products.lock().await;
        lock.insert(id.clone(), product.clone());
        self.product_repository.create(id, product).await
    }

    pub async fn commit(&self) -> Result<(), String> {
        println!("Committing changes");
        let mut lock = self.new_products.lock().await;
        lock.clear();
        Ok(())
    }

    pub async fn rollback(&self) -> Result<(), String> {
        println!("Rolling back changes");
        let mut lock = self.new_products.lock().await;
        lock.clear();
        Ok(())
    }
}