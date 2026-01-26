#[tokio::main]
async fn main() -> Result<(), merc_error::Error> {
    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://admin:admin@localhost:5672".to_string());

    println!("connecting to rabbitmq at {}", rabbitmq_url);

    let mut consumer = merc_events::connect(&rabbitmq_url)
        .await?
        .consume("message.create")
        .await?;

    println!("waiting for messages on memory.create...");

    while let Some(res) = consumer.next::<String>().await {
        let _ = match res {
            Err(err) => return Err(err),
            Ok(v) => v,
        };
    }

    Ok(())
}
