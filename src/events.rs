use amqprs::{callbacks::{DefaultChannelCallback, DefaultConnectionCallback}, channel::{BasicPublishArguments, QueueBindArguments, QueueDeclareArguments}, connection::{Connection, OpenConnectionArguments}, BasicProperties};
use serde::Serialize;

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
}


// event brokers
pub struct RabbitMqMessageBroker{
    connection: Connection,
}

impl RabbitMqMessageBroker{
    pub async fn new(init_info: RabbitMqInitializationInfo) -> Result<RabbitMqMessageBroker, String>{
        
        match Connection::open(&OpenConnectionArguments::new(
            &init_info.uri,
            init_info.port,
            &init_info.username,
            &init_info.password
        )).await {
            Ok(c) => {
                match c.register_callback(DefaultConnectionCallback).await{
                    Ok(_) => Ok(RabbitMqMessageBroker {
                        connection: c
                    }),
                    Err(e) => Err(format!("Failed to register defaut connection callback: {}", e))
                }
            },
            Err(e) => {
                Err(format!("Failed to open RabbitMQ connection: {}", e))
            }
        }
    }
}

impl MessageBroker for RabbitMqMessageBroker{
    async fn publish_message(&self, event: &Event, destination_name: &str) -> Result<(), String> {
        match self.connection.open_channel(None).await{
            Ok(channel) => match channel.register_callback(DefaultChannelCallback).await {
                Ok(_) => match channel.queue_declare(QueueDeclareArguments::default()).await{
                    Ok(queue_option) => match queue_option {
                        Some(q) => match channel.queue_bind(QueueBindArguments::new(&q.0, destination_name, "")).await {
                            Ok(_) => {
                                match serde_json::to_string(&event){
                                    Ok(x) => match channel.basic_publish(BasicProperties::default(), x.into_bytes(), BasicPublishArguments::new(destination_name, "")).await {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(format!("Failed to publish event to broker: {}", e))
                                    }, 
                                    Err(e) => Err(format!("Failed to serialize event: {}", e))
                                }
                            },
                            Err(e) => Err(format!("Failed to bind to queue: {}", e))
                        },
                        None => Err(format!("Failed to declare queue"))
                    },
                    Err(e) => Err(format!("Failed to declare queue: {}", e))
                },
                Err(e) => Err(format!("Failed to register channel callback: {}", e))
            },
            Err(e) => Err(format!("Failed to open channel: {}", e))
        }
    }
}