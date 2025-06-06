use crate::domain::Product;
use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::{bson::doc, Client, ClientSession, Collection};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct MongoDbInitializationInfo {
    pub uri: String,
    pub database: String,
    pub collection: String,
}

#[async_trait]
pub trait ProductRepository {
    async fn create(
        &self,
        id: String,
        product: Product,
        session: Arc<Mutex<ClientSession>>,
    ) -> Result<Product, String>;
    async fn read<'a>(&self, id: &'a str) -> Result<Product, String>;
    async fn read_all(&self) -> Result<Vec<Product>, String>;
    async fn update(
        &self,
        id: String,
        product: Product,
        session: Arc<Mutex<ClientSession>>,
    ) -> Result<Product, String>;
    async fn delete(&self, id: &str, session: Arc<Mutex<ClientSession>>);
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

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn create(
        &self,
        id: String,
        product: Product,
        _: Arc<Mutex<ClientSession>>,
    ) -> Result<Product, String> {
        let mut lock = self.products.lock().await;
        lock.insert(id.clone(), product.clone());
        match lock.get(id.as_str()) {
            Some(x) => Ok(x.clone()),
            None => Err(format!("Product with id {} did not exist", id)),
        }
    }

    async fn read<'a>(&self, id: &'a str) -> Result<Product, String> {
        let lock = self.products.lock().await;
        match lock.get(id) {
            Some(x) => Ok(x.clone()),
            None => Err(format!("Product with id {} did not exist", id)),
        }
    }

    async fn read_all(&self) -> Result<Vec<Product>, String> {
        let mut products_to_return = Vec::new();
        let lock = self.products.lock().await;

        for (_, value) in lock.iter() {
            products_to_return.push(value.clone());
        }

        Ok(products_to_return)
    }

    async fn update(
        &self,
        id: String,
        product: Product,
        _: Arc<Mutex<ClientSession>>,
    ) -> Result<Product, String> {
        let mut lock = self.products.lock().await;
        lock.insert(id.clone(), product.clone());
        match lock.get(id.as_str()) {
            Some(x) => Ok(x.clone()),
            None => Err(format!("Product with id {} did not exist", id)),
        }
    }

    async fn delete(&self, id: &str, _: Arc<Mutex<ClientSession>>) {
        let mut lock = self.products.lock().await;
        lock.remove_entry(id);
    }
}

#[derive(Clone)]
pub struct MongoDbProductRepository {
    product_collection: Collection<Product>,
}

impl MongoDbProductRepository {
    pub async fn new(info: &MongoDbInitializationInfo, client: &Client) -> Self {
        let database = client.database(&info.database);

        MongoDbProductRepository {
            product_collection: database.collection(&info.collection),
        }
    }
}

#[async_trait]
impl ProductRepository for MongoDbProductRepository {
    async fn create(
        &self,
        id: String,
        product: Product,
        session: Arc<Mutex<ClientSession>>,
    ) -> Result<Product, String> {
        let mut guard = session.lock().await;

        match self
            .product_collection
            .insert_one(product)
            .session(&mut *guard)
            .await
        {
            Ok(_) => match self
                .product_collection
                .find_one(doc! {"id": &id})
                .session(&mut *guard)
                .await
            {
                Ok(find_one_product_option) => match find_one_product_option {
                    Some(p) => Ok(p),
                    None => Err(format!("Failed to find product with id {}", id)),
                },
                Err(e) => Err(format!("Failed to insert product: {}", e)),
            },
            Err(e) => Err(format!("Failed to insert product: {}", e)),
        }
    }

    async fn read<'a>(&self, id: &'a str) -> Result<Product, String> {
        match self.product_collection.find_one(doc! {"id": &id}).await {
            Ok(find_one_product_option) => match find_one_product_option {
                Some(p) => Ok(p),
                None => Err(format!("Failed to find product with id {}", id)),
            },
            Err(e) => Err(format!("Failed to insert product: {}", e)),
        }
    }

    async fn read_all(&self) -> Result<Vec<Product>, String> {
        let mut products_to_return = Vec::new();

        match self.product_collection.find(doc! {}).await {
            Ok(mut found_products) => {
                while let Ok(Some(product)) = found_products.try_next().await {
                    products_to_return.push(product.clone())
                }

                Ok(products_to_return)
            }
            Err(_) => Err(format!("Failed to find products")),
        }
    }

    async fn update(
        &self,
        id: String,
        product: Product,
        session: Arc<Mutex<ClientSession>>,
    ) -> Result<Product, String> {
        let mut guard = session.lock().await;

        match self
            .product_collection
            .replace_one(doc! {"id": &id}, product)
            .session(&mut *guard)
            .await
        {
            Ok(_) => match self
                .product_collection
                .find_one(doc! {"id": &id})
                .session(&mut *guard)
                .await
            {
                Ok(find_one_product_option) => match find_one_product_option {
                    Some(p) => Ok(p),
                    None => Err(format!("Failed to find Product with id {}", id)),
                },
                Err(e) => Err(format!("Failed to update Product: {}", e)),
            },
            Err(e) => Err(format!("Failed to update Product: {}", e)),
        }
    }

    async fn delete(&self, id: &str, session: Arc<Mutex<ClientSession>>) {
        todo!()
    }
}
