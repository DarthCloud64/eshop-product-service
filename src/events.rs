use std::sync::Arc;

use amqprs::{callbacks::{DefaultChannelCallback, DefaultConnectionCallback}, channel::{self, BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeBindArguments, ExchangeDeclareArguments, ExchangeType, QueueBindArguments, QueueDeclareArguments}, connection::{Connection, OpenConnectionArguments}, consumer::{AsyncConsumer, DefaultConsumer}, BasicProperties, Deliver, DELIVERY_MODE_PERSISTENT};
use async_trait::async_trait;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Notify};
use tracing::{event, Level};

use crate::{cqrs::{CommandHandler, DecrementProductInventoryCommand, GetProductsQuery, IncrementProdcuctInventoryCommand, QueryHandler}, dtos::GetProductsResponse, state::AppState};

pub static PRODUCT_ADDED_TO_CART_QUEUE_NAME: &str = "product.added.to.cart";
pub static PRODUCT_REMOVED_FROM_CART_QUEUE_NAME: &str = "product.removed.from.cart";

pub struct RabbitMqInitializationInfo{
    uri: String,
    port: u16,
    username: String,
    password: String
}

impl RabbitMqInitializationInfo {
    pub fn new(
        uri: String,
        port: u16,
        username: String,
        password: String) -> RabbitMqInitializationInfo{
            RabbitMqInitializationInfo {
                uri: uri,
                port: port,
                username: username,
                password: password
            }
        }
}

// events
#[derive(Serialize, Deserialize)]
pub enum Event{
    ProductCreatedEvent{
        id: String,
        name: String,
        price: f32
    },
    ProductAddedToCartEvent {
        product_id: String
    },
    ProductRemovedFromCartEvent {
        product_id: String
    }
}

pub trait MessageBroker{
    async fn publish_message(&self, event: &Event, destination_name: &str) -> Result<(), String>;
    async fn consume(&self, source_queue_name: &'static str, state: Arc<AppState>);
}


// event brokers
pub struct RabbitMqMessageBroker{
    connection: Connection,
}

impl RabbitMqMessageBroker{
    pub async fn new(init_info: RabbitMqInitializationInfo) -> Result<RabbitMqMessageBroker, String>{
        match Connection::open(&OpenConnectionArguments::new(&init_info.uri, init_info.port, &init_info.username, &init_info.password)
        ).await {
            Ok(connection) => {
                match connection.register_callback(DefaultConnectionCallback).await {
                    Ok(()) => {
                        Ok(RabbitMqMessageBroker{
                            connection: connection
                        })
                    },
                    Err(e) => {
                        Err(format!("Failed to register connection callback: {}", e))
                    }
                }
            },
            Err(e) => {
                Err(format!("Failed to open RabbitMQ connection: {}", e))
            }
        }
    }

    pub async fn get_channel(&self, destination: &str) -> Result<Channel, String>{
        match self.connection.open_channel(None).await{
            Ok(channel) => {
                channel.register_callback(DefaultChannelCallback).await.unwrap();
                channel.exchange_declare(ExchangeDeclareArguments::new(destination, &ExchangeType::Fanout.to_string())).await.unwrap();
                channel.queue_declare(QueueDeclareArguments::durable_client_named(destination)).await.unwrap();
                channel.queue_bind(QueueBindArguments::new(destination, destination, "")).await.unwrap();

                Ok(channel)
            },
            Err(e) => {
                Err(format!("Failed to get channel: {}", e))
            }
        }
    }
}

impl MessageBroker for RabbitMqMessageBroker{
    async fn publish_message(&self, event: &Event, destination_name: &str) -> Result<(), String> {
        match self.get_channel(destination_name).await{
            Ok(channel) => {
                let mut delivery_properties = BasicProperties::default();
                delivery_properties.with_delivery_mode(DELIVERY_MODE_PERSISTENT);
                match serde_json::to_string(&event){
                    Ok(x) => {
                        event!(Level::DEBUG, "publishing!!! {}", x);
                        match channel.basic_publish(delivery_properties, x.into_bytes(), BasicPublishArguments::new(destination_name, "")).await {
                            Ok(_) => Ok(()),
                            Err(e) => Err(format!("Failed to publish event to broker: {}", e))
                        }
                    }, 
                    Err(e) => Err(format!("Failed to serialize event: {}", e))
                }
            },
            Err(e) => {
                Err(format!("Failed to publish event to broker: {}", e))   
            }
        }
    }

    async fn consume(&self, source_queue_name: &'static str, state: Arc<AppState>) {
        match self.get_channel(source_queue_name).await {
            Ok(channel) => {
                let consume_arguments = BasicConsumeArguments::new(source_queue_name, "eshop-prodct-service")
                    .manual_ack(false)
                    .finish();

                match source_queue_name {
                    queue_name if queue_name == PRODUCT_ADDED_TO_CART_QUEUE_NAME => {
                        channel.basic_consume(ProductAddedToCartEventHandler::new(Mutex::new(state.clone())), consume_arguments).await.unwrap();
                    },
                    queue_name if queue_name == PRODUCT_REMOVED_FROM_CART_QUEUE_NAME => {
                        channel.basic_consume(ProductRemoveFromCartEventHandler::new(Mutex::new(state.clone())), consume_arguments).await.unwrap();
                    }
                    x => event!(Level::INFO, "event {} is not valid to subscribe to", x)
                }

                let guard = Notify::new();
                guard.notified().await;
            },
            Err(e) => {
                panic!();
            }
        }
    }
}

pub struct ProductAddedToCartEventHandler {
    state: Mutex<Arc<AppState>>
}

impl ProductAddedToCartEventHandler {
    pub fn new(state: Mutex<Arc<AppState>>) -> Self{
        ProductAddedToCartEventHandler{
            state
        }
    }
}

#[async_trait]
impl AsyncConsumer for ProductAddedToCartEventHandler {
    async fn consume(
        &mut self,
        _: &Channel,
        _: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ){
        let state_lock = self.state.lock().await;

        let raw_event = String::from_utf8(content).unwrap();
        event!(Level::DEBUG, "Received event: {}", raw_event);

        match serde_json::from_str::<Event>(&raw_event) {
            Ok(deserialized_event) => {
                match deserialized_event {
                    Event::ProductAddedToCartEvent { product_id } => {
                        let decrement_product_inventory_command: DecrementProductInventoryCommand = DecrementProductInventoryCommand { 
                            product_id: product_id
                        };

                        let _ = state_lock.decrement_product_inventory_command_handler.handle(&decrement_product_inventory_command).await;
                    },
                    _ => event!(Level::INFO, "Event not supported")
                }
            },
            Err(e) => {
                event!(Level::WARN, "Failed to deserialize event {}: {}", raw_event, e);
            }
        }
    }
}

pub struct ProductRemoveFromCartEventHandler {
    state: Mutex<Arc<AppState>>
}

impl ProductRemoveFromCartEventHandler {
    pub fn new(state: Mutex<Arc<AppState>>) -> Self {
        ProductRemoveFromCartEventHandler { 
            state: state
        }
    }
}

#[async_trait]
impl AsyncConsumer for ProductRemoveFromCartEventHandler {
    async fn consume(
        &mut self,
        _: &Channel,
        _: Deliver,
        _: BasicProperties,
        content: Vec<u8>,
    ){
        let state_lock = self.state.lock().await;

        let raw_event = String::from_utf8(content).unwrap();
        event!(Level::DEBUG, "Received event: {}", raw_event);

        match serde_json::from_str::<Event>(&raw_event) {
            Ok(deserialized_event) => {
                match deserialized_event {
                    Event::ProductRemovedFromCartEvent {product_id} => {
                        let increment_product_inventory_command = IncrementProdcuctInventoryCommand {
                            product_id: product_id
                        };
        
                        let _ = state_lock.increment_product_inventory_command_handler.handle(&increment_product_inventory_command).await.unwrap();
                    },
                    _ => event!(Level::INFO, "event not supported")
                }
            },
            Err(e) => {
                event!(Level::WARN, "Failed to deserialize event {}: {}", raw_event, e);
            }
        }
    }
}