use futures_lite::StreamExt;
use lapin::{
    Connection, ConnectionProperties,
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://admin:admin@localhost:5672".to_string());

    println!("connecting to rabbitmq at {}", rabbitmq_url);

    let conn = Connection::connect(&rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    println!("connected to rabbitmq");

    let queue = channel
        .queue_declare(
            "memory.create",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("declared queue: {}", queue.name());

    let mut consumer = channel
        .basic_consume(
            "memory.create",
            "merc-worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("waiting for messages on memory.create...");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let data = String::from_utf8_lossy(&delivery.data);
                println!("received message: {}", data);

                // TODO: Process the memory.create message here

                delivery.ack(BasicAckOptions::default()).await?;
            }
            Err(e) => {
                eprintln!("error receiving message: {}", e);
            }
        }
    }

    Ok(())
}
