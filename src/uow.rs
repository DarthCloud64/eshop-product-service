use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{domain::Product, events::{Event, MessageBroker}, repositories::ProductRepository};

#[derive(Clone)]
pub struct RepositoryContext<T1: ProductRepository, T2: MessageBroker> {
    pub product_repository: Arc<T1>,
    message_broker: Arc<T2>,
    new_products: Arc<Mutex<HashMap<String, Product>>>,
    events_to_publish: Arc<Mutex<Vec<Event>>>
}

impl<T1: ProductRepository, T2: MessageBroker> RepositoryContext<T1, T2> {
    pub fn new(product_repository: Arc<T1>, message_broker: Arc<T2>) -> RepositoryContext<T1, T2>{
        RepositoryContext {
            product_repository: product_repository,
            message_broker: message_broker,
            new_products: Arc::new(Mutex::new(HashMap::new())),
            events_to_publish: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_product(&self, id: String, product: Product) -> Result<Product, String>{
        let mut lock = self.new_products.lock().await;
        lock.insert(id.clone(), product.clone());

        let mut lock = self.events_to_publish.lock().await;
        lock.push(Event::ProductCreatedEvent { id: product.id.clone() });

        self.product_repository.create(id, product).await
    }

    pub async fn commit(&self) -> Result<(), String> {
        println!("Committing changes");
        let mut lock = self.new_products.lock().await;
        lock.clear();

        let lock = self.events_to_publish.lock().await;
        for e in lock.iter(){
            self.message_broker.publish_message(e, "product.created").await;
        }
        Ok(())
    }

    pub async fn rollback(&self) -> Result<(), String> {
        println!("Rolling back changes");
        let mut lock = self.new_products.lock().await;
        lock.clear();
        Ok(())
    }
}