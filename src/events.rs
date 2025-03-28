use amqprs::{callbacks::{DefaultChannelCallback, DefaultConnectionCallback}, channel::{self, BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeBindArguments, ExchangeDeclareArguments, ExchangeType, QueueBindArguments, QueueDeclareArguments}, connection::{Connection, OpenConnectionArguments}, consumer::{AsyncConsumer, DefaultConsumer}, BasicProperties, Deliver, DELIVERY_MODE_PERSISTENT};
use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::Notify;
use tracing::{event, Level};

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
#[derive(Serialize)]
pub enum Event{
    ProductCreatedEvent{
        id: String
    }
}

pub trait MessageBroker{
    async fn publish_message(&self, event: &Event, destination_name: &str) -> Result<(), String>;
    async fn consume(&self, destination_name: &str);
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

    async fn consume(&self, destination_name: &str) {
        match self.get_channel(destination_name).await {
            Ok(channel) => {
                let consume_arguments = BasicConsumeArguments::new(destination_name, "eshop-prodct-service")
                    .manual_ack(false)
                    .finish();

                let x = channel.basic_consume(ProductAddedToCartEventHandler::new(), consume_arguments).await.unwrap();
                event!(Level::DEBUG, "Received event: {}", x);

                let guard = Notify::new();
                guard.notified().await;
            },
            Err(e) => {
                panic!();
            }
        }
    }
}

pub struct ProductAddedToCartEventHandler {}

impl ProductAddedToCartEventHandler {
    pub fn new() -> Self{
        ProductAddedToCartEventHandler{}
    }
}

#[async_trait]
impl AsyncConsumer for ProductAddedToCartEventHandler {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ){
        let x = String::from_utf8(content).unwrap();
        event!(Level::DEBUG, "Received event: {}", x);
    }
}