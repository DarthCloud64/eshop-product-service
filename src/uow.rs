use std::sync::Arc;

use async_trait::async_trait;
use mongodb::ClientSession;
use tokio::sync::Mutex;
use tracing::{event, Level};

use crate::{
    events::{Event, MessageBroker},
    repositories::ProductRepository,
};

#[async_trait]
pub trait UnitOfWork {
    async fn get_product_repository(&self) -> Arc<dyn ProductRepository + Send + Sync>;
    async fn begin_transaction(&self) -> Arc<Mutex<ClientSession>>;
    async fn commit(&self) -> Result<(), String>;
    async fn rollback(&self) -> Result<(), String>;
}

#[derive(Clone)]
pub struct ProductUnitOfWork {
    product_repository: Arc<dyn ProductRepository + Send + Sync>,
    message_broker: Arc<dyn MessageBroker + Send + Sync>,
    events_to_publish: Arc<Mutex<Vec<Event>>>,
    client_session: Arc<Mutex<ClientSession>>,
}

impl ProductUnitOfWork {
    pub fn new(
        product_repository: Arc<dyn ProductRepository + Send + Sync>,
        message_broker: Arc<dyn MessageBroker + Send + Sync>,
        client_session: Arc<Mutex<ClientSession>>,
    ) -> ProductUnitOfWork {
        ProductUnitOfWork {
            product_repository: product_repository,
            message_broker: message_broker,
            events_to_publish: Arc::new(Mutex::new(Vec::new())),
            client_session: client_session,
        }
    }
}

#[async_trait]
impl UnitOfWork for ProductUnitOfWork {
    async fn get_product_repository(&self) -> Arc<dyn ProductRepository + Send + Sync> {
        self.product_repository.clone()
    }

    async fn begin_transaction(&self) -> Arc<Mutex<ClientSession>> {
        self.client_session
            .lock()
            .await
            .start_transaction()
            .await
            .unwrap();

        self.client_session.clone()
    }

    async fn commit(&self) -> Result<(), String> {
        event!(Level::TRACE, "Committing changes");

        self.client_session
            .lock()
            .await
            .commit_transaction()
            .await
            .unwrap();

        let mut lock = self.events_to_publish.lock().await;
        let mut event_results = Vec::new();
        for e in lock.iter() {
            event!(Level::TRACE, "publishing event");
            event_results.push(
                self.message_broker
                    .publish_message(e, "product.created")
                    .await,
            );
        }

        let mut single_event_failed = false;
        for result in event_results {
            let _ = match result {
                Ok(()) => (),
                Err(e) => {
                    single_event_failed = true;
                    event!(Level::WARN, "event error found! {}", e);
                }
            };
        }

        lock.clear();

        if single_event_failed {
            return Err(String::from("Failed to commit changes."));
        }

        Ok(())
    }

    async fn rollback(&self) -> Result<(), String> {
        self.client_session
            .lock()
            .await
            .abort_transaction()
            .await
            .unwrap();

        Ok(())
    }
}
